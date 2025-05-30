use std::env;
use std::ffi::OsStr;
use std::path::Path;
use tokio::fs::File;
use walkdir::WalkDir;
use dbpf::DBPFFile;

use binrw::{BinRead, BinResult};
use humansize::{DECIMAL, format_size};

use futures::{stream, StreamExt};

use tracing::{error, info, instrument};
use dbpf_utils::application_main;

#[instrument(skip_all, level = "trace")]
async fn get_size(path: &Path) -> BinResult<(usize, usize)> {
    let mut data = File::open(&path).await.unwrap().into_std().await;
    tokio::task::spawn_blocking(move || {
        DBPFFile::read(&mut data).and_then(|header| {
            let size: usize = header.hole_index
                .iter()
                .map(|hole| hole.size as usize)
                .sum();
            Ok((size, header.hole_index.len()))
        })
    }).await.unwrap()
}

#[instrument]
async fn get_path_size(path: &Path) -> Option<usize> {
    match get_size(&path).await {
        Ok((size, holes)) => {
            if size > 0 {
                info!(size = format_size(size, DECIMAL),
                    number = holes, "total holes in file");
            }
            Some(size)
        }
        Err(err) => {
            error!(%err);
            None
        }
    }
}

#[tokio::main]
async fn main() {
    application_main(|| async {
        let (total_size, num_files) = {
            let flattened = stream::iter(env::args_os().skip(1).map(|arg| {
                WalkDir::new(arg).into_iter().map(|entry| async {
                    let path = entry.unwrap().path().to_path_buf();
                    if path.extension() == Some(OsStr::new("package")) {
                        get_path_size(&path).await
                    } else {
                        None
                    }
                })
            }).flatten()).buffer_unordered(16);
            flattened.fold((0, 0), |cur_size, size| async move {
                if let Some(size) = size {
                    (cur_size.0 + size, cur_size.1 + 1)
                } else {
                    cur_size
                }
            }).await
        };

        println!("Total hole size: {} in {num_files} files", format_size(total_size, DECIMAL));
    }).await;
}
