use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use dbpf::DBPFFile;
use binrw::{BinRead, BinResult};
use humansize::{DECIMAL, format_size};
use tokio::fs::File;
use futures::StreamExt;
use tracing::{Instrument, trace_span};
use tracing_subscriber::layer::SubscriberExt;

use dbpf_utils::traverse_dir::get_paths_recursive;

async fn get_size(path: PathBuf) -> BinResult<(usize, usize)> {
    let mut data = File::open(path).instrument(trace_span!("tokio open")).await.unwrap()
        .into_std().instrument(trace_span!("tokio into_std")).await;
    let _span = tracy_client::span!("get_size internal");
    DBPFFile::read(&mut data).and_then(|mut result| {
        let index = result.header.hole_index.get(&mut data)?;
        let size: usize = index
            .iter()
            .map(|hole| hole.size as usize)
            .sum();
        Ok((size, index.len()))
    })
}

#[tracing::instrument(skip(path))]
async fn get_path_size(path: PathBuf) -> (usize, usize) {
    match get_size(path.clone()).instrument(tracing::trace_span!("get_size")).await {
        Ok((size, holes)) => {
            if size > 0 {
                println!("{} {} in {} holes",
                         path.to_string_lossy(),
                         format_size(size, DECIMAL),
                         holes);
            }
            (size, 1)
        }
        Err(err) => {
            eprintln!("Error in {}:", path.to_string_lossy());
            eprintln!("{err}");
            (0, 0)
        }
    }
}

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(tracing_subscriber::registry()
        .with(tracing_tracy::TracyLayer::new()
            .with_stackdepth(16))
        .with(tracing_subscriber::fmt::layer().pretty())
    ).expect("set up the subscriber");

    let _span = tracing_tracy::client::span!("yeee");
    let span = tracing::span!(tracing::Level::TRACE, "main");
    let (total_size, num_files) = {
        let _enter = span.enter();
        let flattened = tokio_stream::iter(env::args_os().skip(1).map(move |arg| {
            let stream = get_paths_recursive(arg.into());
            let mapped = stream.map(|path: PathBuf| async {
                if path.extension() == Some(OsStr::new("package")) {
                    get_path_size(path).await
                } else {
                    (0, 0)
                }
            });
            mapped
        })).flatten_unordered(None).buffer_unordered(16);
        flattened.fold((0, 0), |(size1, num1), (size2, num2)|
            async move { (size1 + size2, num1 + num2) }).await
    };
    println!("Total hole size: {} in {num_files} files", format_size(total_size, DECIMAL));
}
