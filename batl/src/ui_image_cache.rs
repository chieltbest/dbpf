// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{io::BufReader, BinRead};
use dbpf::{
	internal_file::{resource_collection::ResourceData, DecodedFile},
	DBPFFile,
};
use eframe::{
	egui,
	egui::{ColorImage, TextureHandle},
};
use egui_dock::egui::TextureOptions;
use lru::LruCache;
use std::{
	num::NonZeroUsize,
	sync::{Arc, Mutex, Weak},
};
use tokio::fs::File;
use tracing::error;

use crate::texture_finder::TextureId;
use crate::ui_image_cache::LoadingState::Loaded;

#[derive(Copy, Clone, Debug)]
enum LoadingState<T> {
	Loaded(T),
	Loading,
}

impl<T> LoadingState<T> {
	fn get_value(self) -> Option<T> {
		match self {
			Loaded(t) => Some(t),
			LoadingState::Loading => None,
		}
	}
}

pub struct ImageCache {
	cache: Arc<Mutex<LruCache<TextureId, LoadingState<TextureHandle>>>>,
}

impl ImageCache {
	pub fn new(cap: NonZeroUsize) -> Self {
		Self {
			cache: Arc::new(Mutex::new(LruCache::new(cap))),
		}
	}

	async fn fetch_texture(
		cache: Weak<Mutex<LruCache<TextureId, LoadingState<TextureHandle>>>>,
		id: TextureId,
		ctx: egui::Context,
	) {
		match File::open(&id.path).await {
			Err(err) => {
				error!(?err);
			}
			Ok(async_data) => {
				let data = async_data.into_std().await;
				let mut data = BufReader::new(data);
				let mut file = DBPFFile::read(&mut data).unwrap();
				let index = file
					.index
					.iter_mut()
					.find(|entry| {
						entry.type_id == id.tgi.type_id
							&& entry.group_id == id.tgi.group_id
							&& entry.instance_id.id == id.tgi.instance_id
					})
					.unwrap();
				let idata = index.data(&mut data).unwrap();

				let decoded = idata.decoded().unwrap().unwrap();
				if let DecodedFile::ResourceCollection(res) = decoded {
					let ResourceData::Texture(tex) = &mut res.entries.first_mut().unwrap().data
					else {
						return;
					};
					let found_texture = tex.decompress_all().into_iter().find_map(|t| {
						t.into_iter().rev().find_map(|mip| {
							mip.ok().map(|decoded_tex| {
								ctx.load_texture(
									format!("{:?}", id),
									ColorImage::from_rgba_unmultiplied(
										[decoded_tex.width, decoded_tex.height],
										&decoded_tex.data,
									),
									TextureOptions::NEAREST,
								)
							})
						})
					});

					if let Some(tex) = found_texture {
						if let Some(cache) = cache.upgrade() {
							let mut cw = cache.lock().unwrap();
							cw.push(id, Loaded(tex));
						}
					}
				}
			}
		}
	}

	/// get, and if not found, fetch the texture from disk asynchronously, returning an `egui::TextureHandle`
	pub fn get(&mut self, id: &TextureId, ctx: &egui::Context) -> Option<TextureHandle> {
		let mut cache = self.cache.lock().unwrap();
		let o = cache.get(id).cloned();
		if o.is_none() {
			cache.push(id.clone(), LoadingState::Loading);
			// fetch the image concurrently
			tokio::spawn(Self::fetch_texture(
				Arc::downgrade(&self.cache),
				id.clone(),
				ctx.clone(),
			));
		}
		o.and_then(LoadingState::get_value)
	}
}
