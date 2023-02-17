use std::collections::HashMap;
use std::{env, future};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use walkdir::WalkDir;
use dbpf::{DBPFFile, InstanceId};
use dbpf_utils::*;

use binrw::{BinRead, Error};

use futures::{stream, StreamExt};

use tracing::{error, info, info_span, instrument};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::filetypes::DBPFFileType::Known;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct TGI {
    type_id: DBPFFileType,
    group_id: u32,
    instance_id: InstanceId,
}

#[derive(Debug)]
enum GetTGIsError {
    Header(Error),
    Index(DBPFFile, Error),
}

#[instrument(skip_all, level = "trace")]
async fn get_tgis(path: &Path) -> Result<Vec<TGI>, GetTGIsError> {
    let mut data = File::open(&path).await.unwrap().into_std().await;
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


fn main() {
    application_main(|| async {
        let flattened = stream::iter(env::args_os().skip(1).map(|arg| {
            WalkDir::new(arg).sort_by_file_name().into_iter().map(|entry| async {
                let path = entry.unwrap().path().to_path_buf();
                if path.extension() == Some(OsStr::new("package")) {
                    get_path_tgis(&path).await
                } else {
                    None
                }
            })
        }).flatten()).buffered(16);
        let mut tgi_to_file = HashMap::new();
        flattened.for_each(|data| {
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
                                           KnownDBPFFileType::BCONLabels |
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
                    println!("{original:?} --> {path:?}");
                    for tgi in tgis.clone() {
                        println!("{tgi:X?}");
                    }
                    println!();
                    info_span!("conflict", ?path, ?original).in_scope(|| {
                        info!(tgis = format!("{tgis:X?}"), "found");
                    });
                }
            }
            future::ready(())
        }).await;
    });
}
