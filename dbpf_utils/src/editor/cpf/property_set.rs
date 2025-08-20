// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use dbpf::internal_file::cpf::property_set::{Override, PropertySet};
use eframe::{
	egui,
	egui::{DragValue, Grid, Response, Ui},
	glow,
};

use crate::editor::{
	cpf::reference_edit_fn, drag_checkbox_fn, drag_fn, drag_option_fn, Editor, VecEditorState,
	VecEditorStateStorage,
};

impl Editor for Override {
	type EditorState = ();

	fn new_editor(
		&self,
		_context: &egui::Context,
		_gl: &Option<Arc<glow::Context>>,
	) -> Self::EditorState {
	}

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let mut res = ui
			.add(DragValue::new(&mut self.shape))
			.on_hover_text("shape");
		res |= reference_edit_fn("", &mut self.resource, ui);
		res | self.subset.show_editor(&mut 300.0, ui)
	}
}

impl Editor for PropertySet {
	type EditorState = ();

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let ires = Grid::new("PropertySet edit grid")
			.num_columns(2)
			.show(ui, |ui| {
				macro_rules! drag {
					($name:ident) => {
						drag_fn(stringify!($name), &mut self.$name, ui)
					};
				}

				macro_rules! drag_option {
					($name:ident, $default:expr) => {
						drag_option_fn(stringify!($name), &mut self.$name, $default, ui)
					};
				}

				macro_rules! drag_checkbox {
                    ($name:ident, $($c_name:expr),*) => {
                        drag_checkbox_fn(stringify!($name), &mut self.$name, [$($c_name),*], ui)
                    };
                }
				macro_rules! string {
					($name:ident) => {{
						ui.label(stringify!($name));
						let res = self.$name.show_editor(&mut 300.0, ui);
						ui.end_row();
						res
					}};
				}

				let mut res = drag_option!(version, 6);
				res |= drag_option!(product, 0);

				res |= drag_checkbox!(
					age,
					"toddler",
					"child",
					"teen",
					"adult",
					"elder",
					"baby",
					"young adult"
				);
				res |= drag_checkbox!(gender, "female", "male");
				res |= drag!(species);

				res |= drag!(parts);
				res |= drag_checkbox!(
					outfit,
					"hair",
					"face",
					"top",
					"body",
					"bottom",
					"accessory",
					"long tail",
					"upright ears",
					"short tail",
					"floppy ears",
					"long brush tail",
					"short brush tail",
					"spitz tail",
					"brush spitz tail"
				);

				res |= drag!(flags);
				res |= string!(name);
				res |= string!(creator);
				res |= string!(family);
				res |= drag_option!(genetic, 0.0);

				res |= drag_option!(priority, 0);

				// type is a builtin keyword, so use a different name
				ui.label("type");
				res |= self.type_.show_editor(&mut 300.0, ui);
				ui.end_row();

				res |= string!(skintone);
				res |= string!(hairtone);
				res |= drag_checkbox!(
					category,
					"casual 1",
					"casual 2",
					"casual 3",
					"swimwear",
					"sleepwear",
					"formal",
					"underwear",
					"skintone",
					"pregnant",
					"activewear",
					"try on",
					"naked overlay",
					"outerwear"
				);
				res |= drag!(shoe);
				res |= drag!(fitness);

				res |= reference_edit_fn("resource", &mut self.resource, ui);
				res |= reference_edit_fn("shape", &mut self.shape, ui);

				res
			});

		ires.response
			| ires.inner
			| self.overrides.show_editor(
				&mut VecEditorState {
					columns: 3,
					storage: VecEditorStateStorage::Shared(()),
				},
				ui,
			)
	}
}
