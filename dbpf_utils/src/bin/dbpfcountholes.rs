use std::env;
use std::ffi::OsStr;
use std::path::Path;
use tokio::fs::File;
use walkdir::WalkDir;
use dbpf::DBPFFile;

use binrw::{BinRead, BinResult};
use humansize::{DECIMAL, format_size};

use futures::{stream, StreamExt};

use tokio::time::Instant;

async fn get_size(path: impl AsRef<Path>) -> BinResult<(usize, usize)> {
    let mut data = File::open(&path).await.unwrap().into_std().await;
    tokio::task::spawn_blocking(move || {
        DBPFFile::read(&mut data).and_then(|mut result| {
            let index = result.header.hole_index.get(&mut data)?;
            let size: usize = index
                .iter()
                .map(|hole| hole.size as usize)
                .sum();
            Ok((size, index.len()))
        })
    }).await.unwrap()
}

async fn get_path_size(path: impl AsRef<Path>) -> Option<usize> {
    match get_size(&path).await {
        Ok((size, holes)) => {
            if size > 0 {
                println!("{} {} in {} holes",
                         path.as_ref().to_string_lossy(),
                         format_size(size, DECIMAL),
                         holes);
            }
            Some(size)
        }
        Err(err) => {
            eprintln!("Error in {}:", path.as_ref().to_string_lossy());
            eprintln!("{err}");
            None
        }
    }
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let (total_size, num_files) = {
        let flattened = stream::iter(env::args_os().skip(1).map(|arg| {
            WalkDir::new(arg).into_iter().map(|entry| async {
                let path = entry.unwrap().path().to_path_buf();
                if path.extension() == Some(OsStr::new("package")) {
                    get_path_size(path).await
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
    let elapsed = start.elapsed();
    println!("Total hole size: {} in {num_files} files", format_size(total_size, DECIMAL));
    println!("(in {:?})", elapsed);
}
