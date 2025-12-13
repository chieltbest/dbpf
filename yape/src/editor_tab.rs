// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::export_resource::resource_import_overlay;
use crate::{EditorType, OpenResource};
use dbpf_utils::editor::Editor;
use eframe::egui::{ScrollArea, Ui};
use eframe::glow;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::{Read, Seek};
use std::rc::Weak;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryEditorTab {
	#[serde(skip)]
	pub state: EditorType,
	#[serde(skip)]
	pub data: Weak<RefCell<OpenResource>>,

	pub id: usize,

	// used for (de)serialising
	#[serde(default)]
	pub is_hex_editor: bool,
	#[serde(default)]
	pub index: Option<usize>,
}

impl EntryEditorTab {
	pub fn show<R: Read + Seek>(
		&mut self,
		ui: &mut Ui,
		reader: &mut R,
		gl_ctx: &Option<Arc<glow::Context>>,
	) {
		if let Some(data) = self.data.upgrade() {
			let mut data_ref = data.borrow_mut();

			let (_, replaced) =
				resource_import_overlay(ui, &mut data_ref, reader, |ui, data_ref, reader| {
					ui.add_enabled_ui(!data_ref.ui_deleted, |ui| match &mut self.state {
						EditorType::Error(err) => {
							ScrollArea::vertical().show(ui, |ui| {
								ui.label(format!("{err:?}"));
							});
						}
						EditorType::HexEditor(editor) => {
							let data = data_ref.data.data(reader).unwrap().decompressed().unwrap();
							if let Ok(mut str) = String::from_utf8(data.data.clone()) {
								if !self.is_hex_editor {
									ui.centered_and_justified(|ui| {
										ScrollArea::vertical().show(ui, |ui| {
											if ui.code_editor(&mut str).changed() {
												data.data = str.into_bytes();
											}
										})
									});
									return;
								}
							}
							editor.draw_editor_contents(
								ui,
								data,
								|mem, addr| Some(mem.data[addr]),
								|mem, addr, byte| mem.data[addr] = byte,
							);
						}
						EditorType::DecodedEditor(state) => {
							let decoded = data_ref
								.data
								.data(reader)
								.unwrap()
								.decoded()
								.unwrap()
								.unwrap();
							decoded.show_editor(state, ui);
						}
					});
				});

			if replaced {
				let type_id = data_ref.data.type_id;
				self.state = EditorType::new(
					&mut data_ref.data,
					reader,
					type_id,
					self.is_hex_editor,
					ui.ctx(),
					gl_ctx,
				);
			}
		}
	}
}
