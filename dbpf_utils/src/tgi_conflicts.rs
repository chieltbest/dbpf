use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use tokio::fs::File;
use walkdir::WalkDir;
use dbpf::{DBPFFile, InstanceId};

use binrw::{BinRead, Error};
use binrw::io::BufReader;

use futures::{stream, StreamExt};

use tracing::{error, info, info_span, instrument};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::filetypes::DBPFFileType::Known;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct TGI {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    pub instance_id: InstanceId,
}

impl Debug for TGI {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.type_id.properties()
            .map(|prop| prop.name.to_string())
            .unwrap_or_else(|| self.type_id.extension()).as_str())
            .field("group", &self.group_id)
            .field("instance", &self.instance_id.id)
            .finish()
    }
}

#[derive(Debug)]
enum GetTGIsError {
    Header(Error),
    Index(DBPFFile, Error),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TGIConflict {
    pub original: PathBuf,
    pub new: PathBuf,
    pub tgis: Vec<TGI>,
}

#[instrument(skip_all, level = "trace")]
async fn get_tgis(path: &Path) -> Result<Vec<TGI>, GetTGIsError> {
    let data = File::open(&path).await.unwrap().into_std().await;
    let mut data = BufReader::new(data);
    tokio::task::spawn_blocking(move || {
        DBPFFile::read(&mut data)
            .map_err(|err| GetTGIsError::Header(err))
            .and_then(|mut result| {
                let index_res = result.header.index.get(&mut data);
                match index_res {
                    Err(err) => Err(GetTGIsError::Index(result.clone(), err)),
                    Ok(index) => {
                        let tgis = index
                            .iter()
                            .map(|file| TGI {
                                type_id: file.type_id,
                                group_id: file.group_id,
                                instance_id: file.instance_id,
                            })
                            .collect();
                        Ok(tgis)
                    }
                }
            })
    }).await.unwrap()
}

#[instrument(level = "error")]
async fn get_path_tgis(path: &Path) -> Option<(PathBuf, Vec<TGI>)> {
    match get_tgis(&path).await {
        Ok(tgis) => {
            Some((path.to_path_buf(), tgis))
        }
        Err(err) => {
            error!("{err:#?}");
            None
        }
    }
}

pub async fn find_conflicts(dir: PathBuf, tx: Sender<TGIConflict>) {
    let mut tgis_stream = stream::iter(WalkDir::new(dir).sort_by_file_name().into_iter().map(|entry| async {
        let path = entry.unwrap().path().to_path_buf();
        if path.extension() == Some(OsStr::new("package")) {
            get_path_tgis(&path).await
        } else {
            None
        }
    })).buffered(1000);

    let mut tgi_to_file = HashMap::new();

    while let Some(data) = tgis_stream.next().await {
        if let Some((path, tgis)) = data {
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
