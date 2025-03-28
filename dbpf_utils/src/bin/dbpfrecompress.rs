use binrw::{BinRead, BinResult};
use clap::Parser;
use dbpf::internal_file::CompressionError;
use dbpf::{CompressionType, DBPFFile};
use dbpf_utils::application_main;
use futures::stream::FuturesOrdered;
use futures::{stream, StreamExt, TryStreamExt};
use humansize::FormatSizeOptions;
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version, about = "Recompress DBPF .package files")]
struct Args {
    #[arg(short, long)]
    decompress: bool,

    #[arg(required = true)]
    file_or_directory: Vec<PathBuf>,
}

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    CompressionError(#[from] CompressionError),
}

async fn compress_file(path: PathBuf, decompress: bool) -> Result<(usize, usize), Error> {
    let data = tokio::fs::read(&path).await?;
    let in_bytes_len = data.len();

    let mut cursor = Cursor::new(data);
    let mut write_cursor = cursor.clone();

    let mut file = DBPFFile::read(&mut cursor).map_err(|err| CompressionError::from(err))?;
    file.index
        .iter_mut()
        .map(|entry| entry.data(&mut cursor))
        .collect::<BinResult<Vec<_>>>()
        .map_err(|err| CompressionError::from(err))?;

    let entries = std::mem::take(&mut file.index)
        .into_iter()
        .map(async |mut entry| {
            let decompress = decompress.clone();
            tokio_rayon::spawn(move || {
                let compression = if decompress {
                    CompressionType::Uncompressed
                } else {
                    CompressionType::RefPack
                };
                let mut cur = Cursor::new(vec![]);
                entry.compression = compression;
                let data = entry.data(&mut cur)?;
                data.decompressed()?;
                data.compressed(compression)?;
                Ok::<_, CompressionError>(entry)
            })
            .await
        })
        .collect::<FuturesOrdered<_>>()
        .try_collect::<Vec<_>>()
        .await?;

    file.index = entries;

    let mut out_buf = Cursor::new(vec![]);
    file.write(&mut out_buf, &mut write_cursor)?;
    let out_bytes = out_buf.into_inner();
    tokio::fs::write(path, &out_bytes).await?;
    Ok((in_bytes_len, out_bytes.len()))
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    application_main(|| async {
        let flattened = stream::iter(
            args.file_or_directory
                .into_iter()
                .map(|arg| {
                    WalkDir::new(arg).into_iter().filter_map(|entry| {
                        let path = entry.unwrap().path().to_path_buf();
                        if path.extension() == Some(OsStr::new("package")) {
                            Some(path)
                        } else {
                            None
                        }
                    })
                })
                .flatten()
                .map(|path| async { (path.clone(), compress_file(path, args.decompress).await) }),
        )
        .buffer_unordered(num_cpus::get());

        let (before, after) = flattened
            .fold((0, 0), |state, item| async move {
                println!("{:?}: {:?}", item.0, item.1);
                if let Ok((before, after)) = item.1 {
                    (state.0 + before, state.1 + after)
                } else {
                    state
                }
            })
            .await;

        println!(
            "Before: {}",
            humansize::format_size(before, FormatSizeOptions::default())
        );
        println!(
            "After: {}",
            humansize::format_size(after, FormatSizeOptions::default())
        );
    })
    .await;
}
