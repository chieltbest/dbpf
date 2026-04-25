// SPDX-FileCopyrightText: 2026 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Write;

use crate::editor::r#enum::{EnumEditor, EnumEditorState};
use crate::editor::vector::VecEditorState;
use crate::editor::{
	cpf::reference_edit_fn, drag_checkbox_fn, drag_hex_fn, drag_option_fn, Editor,
};
use dbpf::internal_file::cpf::property_set::{KnownShoe, Override, PropertySet, Shoe};
use eframe::egui::{DragValue, Grid, Response, Ui};

impl EnumEditor for Shoe {
	type KnownEnum = KnownShoe;

	fn from_known(known_enum: &Self::KnownEnum) -> Self {
		Self::Known(*known_enum)
	}

	fn from_string(string: &String) -> Option<Self>
	where
		Self: Sized,
	{
		u32::from_str_radix(string.trim_start_matches("0x"), 16)
			.map(Shoe::from)
			.ok()
	}

	fn known_name(known_enum: &Self::KnownEnum) -> String {
		format!("{known_enum:?}")
	}

	fn full_name(&self) -> String {
		match self {
			Shoe::Known(known) => Self::known_name(known),
			Shoe::Unknown(i) => format!("{i}"),
		}
	}

	fn known_hover_string(known_enum: &Self::KnownEnum) -> String {
		let mut str = Self::known_name(known_enum);
		write!(str, "\n0x{:X}", *known_enum as u32).unwrap();
		str
	}

	fn hover_string(&self) -> Option<String> {
		match self {
			Shoe::Known(known) => Some(Self::known_hover_string(known)),
			Shoe::Unknown(_) => None,
		}
	}

	fn search_strings(known_enum: &Self::KnownEnum) -> Vec<String> {
		vec![
			format!("{known_enum:?}"),
			format!("{:08X}", *known_enum as u32),
		]
	}

	fn all_known() -> impl Iterator<Item = Self::KnownEnum> {
		enum_iterator::all()
	}
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PropertySetEditorState(EnumEditorState);

impl Editor for Override {
	type EditorState = ();

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let mut res = ui
			.add(DragValue::new(&mut self.shape))
			.on_hover_text("shape");
		res |= reference_edit_fn("", &mut self.resource, ui);
		res | self.subset.show_editor(&mut 300.0, ui)
	}
}

impl Editor for PropertySet {
	type EditorState = PropertySetEditorState;

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let ires = Grid::new("PropertySet edit grid")
			.num_columns(2)
			.show(ui, |ui| {
				macro_rules! drag {
					($name:ident) => {
						drag_hex_fn(stringify!($name), &mut self.$name, ui)
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

				res |= drag_checkbox!(flags, "hidden", "hat", "", "townie disabled", "unknown");
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

				ui.label("shoe");
				res |= self.shoe.show_enum_editor(&mut state.0, ui);
				ui.end_row();

				res |= drag!(fitness);

				res |= reference_edit_fn("resource", &mut self.resource, ui);
				res |= reference_edit_fn("shape", &mut self.shape, ui);

				res
			});

		ires.response
			| ires.inner
			| self
				.overrides
				.show_editor(&mut VecEditorState::Shared(()), ui)
	}
}
