use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use binrw::io::BufReader;
use binrw::{BinRead, BinResult};

use futures::{stream, StreamExt};
use tokio::fs::File;
use walkdir::WalkDir;

use dbpf::filetypes::DBPFFileType::Known;
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::DBPFFile;

use tracing::{error, info, info_span, instrument};

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Tgi {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    pub instance_id: u64,
}

impl Debug for Tgi {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.type_id.properties()
            .map(|prop| prop.name.to_string())
            .unwrap_or_else(|| self.type_id.extension()).as_str())
            .field("group", &self.group_id)
            .field("instance", &self.instance_id)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TGIConflict {
    pub original: PathBuf,
    pub new: PathBuf,
    pub tgis: Vec<Tgi>,
}

impl Display for TGIConflict {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} --> {}", self.original.display(), self.new.display())?;
        for tgi in &self.tgis {
            writeln!(f, "{:?}", tgi)?;
        }
        Ok(())
    }
}

#[instrument(skip_all, level = "trace")]
fn get_tgis(header: DBPFFile) -> Vec<Tgi> {
    header.index.iter()
        .map(|file| Tgi {
            type_id: file.type_id,
            group_id: file.group_id,
            instance_id: file.instance_id,
        })
        .collect()
}

#[instrument(level = "error")]
async fn get_path_tgis(path: PathBuf) -> (PathBuf, Option<Vec<Tgi>>) {
    let data = File::open(&path).await.unwrap().into_std().await;
    let mut data = BufReader::new(data);
    let result = tokio::task::spawn_blocking(move || -> BinResult<Vec<Tgi>> {
        Ok(get_tgis(DBPFFile::read(&mut data)?))
    }).await.unwrap();
    match result {
        Ok(tgis) => {
            (path.to_path_buf(), Some(tgis))
        }
        Err(err) => {
            error!("{err:#?}");
            (path, None)
        }
    }
}

pub async fn find_conflicts(dirs: Vec<PathBuf>,
                            tx: Sender<TGIConflict>,
                            mut progress: impl FnMut(PathBuf, usize, usize)) {
    let files_futures_vec = dirs.iter().flat_map(|dir| {
        WalkDir::new(dir).sort_by_file_name().into_iter().filter_map(|entry| {
            let path = entry.unwrap().path().to_path_buf();
            if path.extension() == Some(OsStr::new("package")) {
                Some(get_path_tgis(path.clone()))
            } else {
                None
            }
        })
    }).collect::<Vec<_>>();
    let total_files = files_futures_vec.len();
    progress(PathBuf::from(""), 0, total_files);

    let mut tgis_stream = stream::iter(
        files_futures_vec.into_iter()
    ).buffered(512).enumerate();

    let mut tgi_to_file = HashMap::new();

    while let Some((i, (path, data))) = tgis_stream.next().await {
        progress(path.clone(), i + 1, total_files);
        if let Some(tgis) = data {
            let mut internal_conflict_files: HashMap<PathBuf, Vec<Tgi>> = HashMap::new();
            for tgi in tgis {
                match tgi {
                    // BCON, BHAV, CTSS, GLOB, GZPS, OBJD, OBJF, SLOT, STR, TPRP, TRCN, TTAB, TTAS, VERS
                    Tgi {
                        type_id: Known(KnownDBPFFileType::SimanticsBehaviourConstants |
                                       KnownDBPFFileType::SimanticsBehaviourFunction |
                                       KnownDBPFFileType::CatalogDescription |
                                       KnownDBPFFileType::GlobalData |
                                       KnownDBPFFileType::PropertySet |
                                       KnownDBPFFileType::ObjectData |
                                       KnownDBPFFileType::ObjectFunctions |
                                       KnownDBPFFileType::ObjectSlot |
                                       KnownDBPFFileType::TextList |
                                       KnownDBPFFileType::EdithSimanticsBehaviourLabels |
                                       KnownDBPFFileType::BehaviourConstantsLabels |
                                       KnownDBPFFileType::PieMenuFunctions |
                                       KnownDBPFFileType::PieMenuStrings |
                                       KnownDBPFFileType::VersionInformation),
                        group_id,
                        ..
                    } if group_id != 0xFFFFFFFF => {
                        // if insert finds a conflict it will return Some(path)
                        if let Some(conflict_path) = tgi_to_file.insert(tgi, path.clone()) {
                            // try to find a list of the conflicts in this file for another file
                            if let Some(file_conflicts) = internal_conflict_files.get_mut(&conflict_path) {
                                // append if there was already a list
                                file_conflicts.push(tgi);
                            } else {
                                // otherwise create a new one
                                internal_conflict_files.insert(conflict_path.clone(), vec![tgi]);
                            }
                        }
                    }
                    _ => {}
                }
            }

            for (original, tgis) in internal_conflict_files.drain() {
                if tx.send(TGIConflict {
                    original: original.clone(),
                    new: path.clone(),
                    tgis: tgis.clone(),
                }).is_err() {
                    return;
                }
                info_span!("conflict", ?path, ?original).in_scope(|| {
                    info!(tgis = format!("{tgis:X?}"), "found");
                });
            }
        }
    };
}
