// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::Editor;
use dbpf::header_v1::InstanceId;
use eframe::egui::{DragValue, Response, Ui};

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
