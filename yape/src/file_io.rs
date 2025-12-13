// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use rfd::FileHandle;
use std::io::Error;
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
pub async fn read_file_handle(handle: FileHandle) -> (Vec<u8>, PathBuf) {
	(handle.read().await, handle.path().to_owned())
}

#[cfg(target_arch = "wasm32")]
pub async fn read_file_handle(handle: FileHandle) -> (Vec<u8>, PathBuf) {
	(handle.read().await, PathBuf::default())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn write_file_handle(handle: FileHandle, buf: &[u8]) -> Result<Option<PathBuf>, Error> {
	handle.write(buf).await?;
	Ok(Some(handle.path().to_owned()))
}

#[cfg(target_arch = "wasm32")]
pub async fn write_file_handle(handle: FileHandle, buf: &[u8]) -> Result<Option<PathBuf>, Error> {
	handle.write(buf).await?;
	Ok(None)
}
