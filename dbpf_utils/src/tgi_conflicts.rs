use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use binrw::{BinRead, Error};
use binrw::io::BufReader;

use futures::{stream, StreamExt};
use walkdir::WalkDir;
use tokio::fs::File;

use dbpf::{DBPFFile, Header, Index, IndexEntry};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::filetypes::DBPFFileType::Known;

use tracing::{error, info, info_span, instrument};

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct TGI {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    pub instance_id: u64,
}

impl Debug for TGI {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.type_id.properties()
            .map(|prop| prop.name.to_string())
            .unwrap_or_else(|| self.type_id.extension()).as_str())
            .field("group", &self.group_id)
            .field("instance", &self.instance_id)
            .finish()
    }
}

#[derive(Debug)]
enum GetTGIsError {
    Header(Error),
    Index(Error),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TGIConflict {
    pub original: PathBuf,
    pub new: PathBuf,
    pub tgis: Vec<TGI>,
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
fn get_tgis<R: Read + Seek>(header: &mut impl Header, reader: &mut R) -> Result<Vec<TGI>, GetTGIsError> {
    let index = match header.index(reader) {
        Err(err) => return Err(GetTGIsError::Index(err)),
        Ok(index) => index,
    };
    let tgis = index.entries()
        .iter()
        .map(|file| TGI {
            type_id: file.get_type().clone(),
            group_id: file.get_group(),
            instance_id: file.get_instance(),
        })
        .collect();
    Ok(tgis)
}

#[instrument(level = "error")]
async fn get_path_tgis(path: PathBuf) -> (PathBuf, Option<Vec<TGI>>) {
    let data = File::open(&path).await.unwrap().into_std().await;
    let mut data = BufReader::new(data);
    let result = tokio::task::spawn_blocking(move || {
        DBPFFile::read(&mut data)
            .map_err(|err| GetTGIsError::Header(err))
            .and_then(|mut result| {
                match result {
                    DBPFFile::HeaderV1(ref mut header) => get_tgis(header, &mut data),
                    DBPFFile::HeaderV2(ref mut header) => get_tgis(header, &mut data),
                }
            })
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
    let files_futures_vec = dirs.iter().map(|dir| {
        WalkDir::new(dir).sort_by_file_name().into_iter().filter_map(|entry| {
            let path = entry.unwrap().path().to_path_buf();
            if path.extension() == Some(OsStr::new("package")) {
                Some(get_path_tgis(path.clone()))
            } else {
                None
            }
        })
    }).flatten().collect::<Vec<_>>();
    let total_files = files_futures_vec.len();
    progress(PathBuf::from(""), 0, total_files);

    let mut tgis_stream = stream::iter(
        files_futures_vec.into_iter()
    ).buffered(512).enumerate();

    let mut tgi_to_file = HashMap::new();

    while let Some((i, (path, data))) = tgis_stream.next().await {
        progress(path.clone(), i + 1, total_files);
        if let Some(tgis) = data {
            let mut internal_conflict_files: HashMap<PathBuf, Vec<TGI>> = HashMap::new();
            for tgi in tgis {
                match tgi {
                    // BCON, BHAV, CTSS, GLOB, GZPS, OBJD, OBJF, SLOT, STR, TPRP, TRCN, TTAB, TTAS, VERS
                    TGI {
                        type_id: Known(KnownDBPFFileType::SimanticsBehaviourConstant |
                                       KnownDBPFFileType::SimanticsBehaviourFunction |
                                       KnownDBPFFileType::CatalogDescription |
                                       KnownDBPFFileType::GlobalData |
                                       KnownDBPFFileType::PropertySet |
                                       KnownDBPFFileType::ObjectData |
                                       KnownDBPFFileType::ObjectFunctions |
                                       KnownDBPFFileType::ObjectSlot |
                                       KnownDBPFFileType::TextList |
                                       KnownDBPFFileType::EdithSimanticsBehaviourLabels |
                                       KnownDBPFFileType::BehaviourConstantLabels |
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
                if let Err(_) = tx.send(TGIConflict {
                    original: original.clone(),
                    new: path.clone(),
                    tgis: tgis.clone(),
                }) {
                    return;
                }
                info_span!("conflict", ?path, ?original).in_scope(|| {
                    info!(tgis = format!("{tgis:X?}"), "found");
                });
            }
        }
    };
}
