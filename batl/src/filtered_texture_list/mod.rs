// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use crate::filtered_texture_list::sort_order::{SortOrder, TextureSortOperation, TextureSorter};
use crate::texture_finder::{FoundTexture, TextureId};
use dbpf_utils::editor::Editor;
use eframe::{
	egui::{Ui, Window},
	glow, Storage,
};
use filter_rule::{TextureFilter, TextureFilterRule};

mod filter_rule;
pub mod sort_order;

#[derive(Clone, Default, Debug)]
pub struct FilteredTextureList {
	known_textures: Vec<TextureId>,
	show_known: bool,
	texture_filter: TextureFilter,
	texture_filter_ui_state: Option<<TextureFilter as Editor>::EditorState>,
	open_texture_filter_ui: bool,
	sorter: TextureSorter,

	found_textures: Vec<FoundTexture>,
	filtered_textures: Vec<FoundTexture>,
}

impl FilteredTextureList {
	pub fn new(storage: &Option<&dyn Storage>) -> Self {
		let mut new = Self {
			show_known: true,

			..Default::default()
		};
		if let Some(storage) = storage {
			if let Some(known_textures_str) = storage.get_string("known_textures") {
				if let Ok(vec) = serde_json::from_str(known_textures_str.as_str()) {
					new.known_textures = vec;
				}
			}

			if let Some(show_known) = storage
				.get_string("show_known")
				.and_then(|str| str.parse().ok())
			{
				new.show_known = show_known;
			}

			if let Some(filter) = storage
				.get_string("filter_list")
				.and_then(|str| serde_json::from_str(&str).ok())
			{
				new.texture_filter = filter;
			}

			if let Some(sorter) = storage
				.get_string("sorter")
				.and_then(|str| serde_json::from_str(&str).ok())
			{
				new.sorter = sorter;
			}
		}
		new
	}

	pub fn save(&mut self, storage: &mut dyn Storage) {
		if let Ok(str) = serde_json::to_string(&self.known_textures) {
			storage.set_string("known_textures", str);
		}

		storage.set_string("show_known", self.get_show_known().to_string());

		storage.set_string(
			"filter_list",
			serde_json::to_string(&self.texture_filter).unwrap(),
		);

		storage.set_string("sorter", serde_json::to_string(&self.sorter).unwrap());
	}

	pub fn show_filter_menu(&mut self, ui: &mut Ui, gl: &Option<Arc<glow::Context>>) {
		let res = Window::new("Filter List")
			.resizable(false)
			.open(&mut self.open_texture_filter_ui)
			.show(ui.ctx(), |ui| {
				let state = self
					.texture_filter_ui_state
					.get_or_insert_with(|| self.texture_filter.new_editor(ui.ctx(), gl));
				self.texture_filter.show_editor(state, ui)
			});
		res.map(|r| {
			r.inner.map(|inner| {
				inner.changed().then(|| {
					self.re_filter();
				})
			})
		});

		ui.button("Filter")
			.on_hover_text("The filters that are being applied to the found texture list")
			.clicked()
			.then(|| {
				self.open_texture_filter_ui = !self.open_texture_filter_ui;
			});
	}

	pub fn get_known(&self) -> &Vec<TextureId> {
		&self.known_textures
	}

	pub fn add_known(&mut self, known: TextureId) -> bool {
		for self_known in &self.known_textures {
			if &known == self_known {
				return false;
			}
		}
		self.known_textures.push(known);
		self.re_filter();
		true
	}

	pub fn remove_known(&mut self, i: usize) {
		self.known_textures.remove(i);
		self.re_filter();
	}

	pub fn is_known(&self, found: &FoundTexture) -> bool {
		self.known_textures.contains(&found.id)
	}

	pub fn set_show_known(&mut self, show: bool) {
		self.show_known = show;
		self.re_filter();
	}

	pub fn get_show_known(&self) -> bool {
		self.show_known
	}

	pub fn set_sort(&mut self, sort_order: TextureSortOperation) {
		self.sorter.set_sort(sort_order);
		self.sort();
	}

	pub fn get_sort(&self) -> &TextureSortOperation {
		self.sorter.get_sort()
	}

	pub fn add(&mut self, found: FoundTexture) {
		self.found_textures.push(found.clone());
		if self.filter_texture(&found) {
			let (Ok(idx) | Err(idx)) = self
				.filtered_textures
				.binary_search_by(|txtr| self.sorter.sort_cmp(txtr, &found));
			self.filtered_textures.insert(idx, found);
		}
	}

	fn filter_texture(&self, found: &FoundTexture) -> bool {
		self.texture_filter.filter(found) && (self.show_known || !self.is_known(found))
	}

	pub fn get_filtered(&self) -> &Vec<FoundTexture> {
		&self.filtered_textures
	}

	pub fn clear(&mut self) {
		self.found_textures = Vec::new();
		self.re_filter();
	}

	fn sort(&mut self) {
		self.filtered_textures
			.sort_unstable_by(|txtr1, txtr2| self.sorter.sort_cmp(txtr1, txtr2));
	}

	fn re_filter(&mut self) {
		let mut filtered_textures = std::mem::take(&mut self.filtered_textures);

		filtered_textures.clear();

		filtered_textures.extend(
			self.found_textures
				.iter()
				.filter(|&tex| self.filter_texture(tex))
				.cloned(),
		);

		self.filtered_textures = filtered_textures;

		self.sort();
	}
}
