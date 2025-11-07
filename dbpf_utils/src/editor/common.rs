// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use crate::editor::Editor;
use binrw::NullWideString;
use dbpf::common::{BigString, ByteString, NullString, PascalString};
use dbpf::header_v1::InstanceId;
use eframe::egui::DragValue;
use eframe::{
	egui,
	egui::{Response, TextEdit, Ui, Vec2},
	glow,
};

trait StringEditor: TryInto<String> + From<String> + Clone {}

impl<T: StringEditor> Editor for T {
	type EditorState = f32;

	fn new_editor(
		&self,
		_context: &egui::Context,
		_gl_context: &Option<Arc<glow::Context>>,
	) -> Self::EditorState {
		300.0
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let string_res = self.clone().try_into();
		match string_res {
			Ok(mut str) => {
				let text_edit = TextEdit::singleline(&mut str)
					.min_size(Vec2::new(*state, 0.0))
					.desired_width(*state);
				let res = text_edit.show(ui).response;
				if res.changed() {
					*self = str.into();
				}
				res
			}
			Err(_) => {
				// ui.add_enabled(
				//     false,
				//     TextEdit::singleline(
				//         &mut String::from_utf8_lossy(string.data.as_slice()).to_string()))
				ui.label("non-utf8 string")
			}
		}
	}
}

impl StringEditor for ByteString {}

impl StringEditor for PascalString<u32> {}
impl StringEditor for PascalString<u8> {}

impl StringEditor for BigString {}

impl StringEditor for NullString {}
impl StringEditor for NullWideString {}

impl Editor for InstanceId {
	type EditorState = ();

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		ui.horizontal(|ui| {
			ui.add(
				DragValue::from_get_set(|v| {
					if let Some(v) = v {
						self.id = ((v as u64) << 32) + (self.id & 0xFFFF_FFFF);
					}
					(self.id >> 32) as f64
				})
				.hexadecimal(8, false, true),
			)
			.on_hover_text("resource / instance-high")
				| ui.add(
					DragValue::from_get_set(|v| {
						if let Some(v) = v {
							self.id =
								(self.id & 0xFFFF_FFFF_0000_0000) + ((v as u64) & 0xFFFF_FFFF);
						}
						(self.id & 0xFFFF_FFFF) as f64
					})
					.hexadecimal(8, false, true),
				)
				.on_hover_text("instance / instance-low")
		})
		.inner
	}
}
