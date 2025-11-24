// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use dbpf::internal_file::cpf::binary_index::BinaryIndex;
use eframe::egui::{Grid, Response, Ui};

use crate::editor::{cpf::reference_edit_fn, drag_hex_fn, Editor};

impl Editor for BinaryIndex {
	type EditorState = ();

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let ires = Grid::new("BinaryIndex edit grid")
			.num_columns(2)
			.max_col_width(1000.0)
			.show(ui, |ui| {
				macro_rules! drag {
					($name:ident) => {
						drag_hex_fn(stringify!($name), $name, ui)
					};
				}
				macro_rules! reference {
					($name:ident) => {
						reference_edit_fn(stringify!($name), $name, ui)
					};
				}
				macro_rules! string {
					($name:ident) => {{
						ui.label(stringify!($name));
						let res = $name.show_editor(&mut 300.0, ui);
						ui.end_row();
						res
					}};
				}

				let BinaryIndex {
					icon,
					stringset,
					bin,
					object,
					creatorid,
					sortindex,
					stringindex,
				} = self;

				let mut res = reference!(icon);
				res |= reference!(stringset);
				res |= reference!(bin);
				res |= reference!(object);
				res |= string!(creatorid);
				res |= drag!(sortindex);
				res |= drag!(stringindex);

				res
			});

		ires.response | ires.inner
	}
}
