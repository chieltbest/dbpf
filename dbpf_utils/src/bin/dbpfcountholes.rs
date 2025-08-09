use std::{env, ffi::OsStr, path::Path};

use binrw::{BinRead, BinResult};
use dbpf::DBPFFile;
use dbpf_utils::application_main;
use futures::{stream, StreamExt};
use humansize::{format_size, DECIMAL};
use tokio::fs::File;
use tracing::{error, info, instrument};
use walkdir::WalkDir;

#[instrument(skip_all, level = "trace")]
async fn get_size(path: &Path) -> BinResult<(usize, usize)> {
	let mut data = File::open(&path).await?.into_std().await;
	tokio::task::spawn_blocking(move || {
		DBPFFile::read(&mut data).map(|header| {
			let size: usize = header
				.hole_index
				.iter()
				.map(|hole| hole.size as usize)
				.sum();
			(size, header.hole_index.len())
		})
	})
	.await
	.unwrap()
}

#[instrument]
async fn get_path_size(path: &Path) -> Option<usize> {
	match get_size(path).await {
		Ok((size, holes)) => {
			if size > 0 {
				info!(
					size = format_size(size, DECIMAL),
					number = holes,
					"total holes in file"
				);
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
			let flattened = stream::iter(env::args_os().skip(1).flat_map(|arg| {
				WalkDir::new(arg).into_iter().map(|entry| async {
					let path = entry.unwrap().path().to_path_buf();
					if path.extension() == Some(OsStr::new("package")) {
						get_path_size(&path).await
					} else {
						None
					}
				})
			}))
			.buffer_unordered(16);
			flattened
				.fold((0, 0), |cur_size, size| async move {
					if let Some(size) = size {
						(cur_size.0 + size, cur_size.1 + 1)
					} else {
						cur_size
					}
				})
				.await
		};

		println!(
			"Total hole size: {} in {num_files} files",
			format_size(total_size, DECIMAL)
		);
	})
	.await;
}
