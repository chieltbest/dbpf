use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use async_stream::stream;
use futures::{StreamExt, TryFutureExt};
use tokio::fs::DirEntry;
use tokio_stream::{Stream};
use tokio_stream::wrappers::ReadDirStream;

pub fn stream_future_unwrap<T, S: Stream<Item=T>, Fut: Future<Output=S>>(fut: Fut) -> impl Stream<Item=T> {
    stream! {
        for await item in fut.await {
            yield item;
        }
    }
}

/// Recursively traverse a directory and return all files in the directory and its subdirectories
// #[tracing::instrument(skip(path))]
pub fn traverse_dir(path: DirEntry) -> Pin<Box<dyn Stream<Item=PathBuf> + Send + 'static>> {
    Box::pin(stream! {
        yield path.path();
        if path.file_type().await.map_or(false, |ft| ft.is_dir()) {
            let stream = ReadDirStream::new(tokio::fs::read_dir(path.path()).await.unwrap());
            let mut mapped_stream = stream.flat_map_unordered(None, move |entry| {
                traverse_dir(entry.unwrap())
            });
            // flat_map_unordered takes care of concurrency, so waiting in a synchronous way is okay here
            while let Some(item) = mapped_stream.next().await {
                yield item;
            }
        }
    })
}

pub fn get_paths_recursive(path: PathBuf) -> Pin<Box<dyn Stream<Item=PathBuf> + Send + 'static>> {
    if path.is_dir() {
        tokio::fs::read_dir(path)
            .and_then(|rd| async { Ok(ReadDirStream::new(rd)) })
            .try_flatten_stream()
            .flat_map_unordered(None, move |entry| {
                traverse_dir(entry.unwrap())
            }).boxed()
    } else if path.is_file() {
        tokio_stream::once(path.to_path_buf()).boxed()
    } else {
        tokio_stream::empty().boxed()
    }
}
