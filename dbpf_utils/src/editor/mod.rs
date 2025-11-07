// SPDX-FileCopyrightText: 2023-2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fmt::Debug, sync::Arc};

use crate::editor::resource_collection::ResourceCollectionEditorState;
use dbpf::{
	filetypes::{DBPFFileType, KnownDBPFFileType},
	internal_file::{
		behaviour::behaviour_function::BehaviourFunction, sim_outfits::SimOutfits,
		text_list::TextList, DecodedFile,
	},
};
use eframe::{
	egui,
	egui::{Align, DragValue, Response, Ui},
	emath::Numeric,
	glow,
};

pub mod behaviour_function;
pub mod common;
pub mod cpf;
pub mod r#enum;
pub mod header;
pub mod object_data;
pub mod resource_collection;
pub mod sim_outfits;
pub mod text_list;
pub mod vector;

pub trait Editor {
	type EditorState: Default;

	fn new_editor(
		&self,
		_context: &egui::Context,
		_gl_context: &Option<Arc<glow::Context>>,
	) -> Self::EditorState {
		Self::EditorState::default()
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response;
}

#[derive(Debug, Default)]
pub enum DecodedFileEditorState {
	ResourceCollection(ResourceCollectionEditorState),
	SimOutfits(<SimOutfits as Editor>::EditorState),
	TextList(<TextList as Editor>::EditorState),
	BehaviourFunction(<BehaviourFunction as Editor>::EditorState),
	#[default]
	None,
}

impl Editor for DecodedFile {
	type EditorState = DecodedFileEditorState;

	fn new_editor(
		&self,
		_context: &egui::Context,
		gl_context: &Option<Arc<glow::Context>>,
	) -> Self::EditorState {
		match self {
			DecodedFile::SimOutfits(skin) => {
				DecodedFileEditorState::SimOutfits(skin.new_editor(_context, gl_context))
			}
			DecodedFile::ResourceCollection(rcol) => {
				DecodedFileEditorState::ResourceCollection(rcol.new_editor(_context, gl_context))
			}
			DecodedFile::TextList(str) => {
				DecodedFileEditorState::TextList(str.new_editor(_context, gl_context))
			}
			DecodedFile::BehaviourFunction(bhav) => {
				DecodedFileEditorState::BehaviourFunction(bhav.new_editor(_context, gl_context))
			}
			_ => DecodedFileEditorState::None,
		}
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		match (self, state) {
			(DecodedFile::PropertySet(gzps), _) => gzps.show_editor(&mut (), ui),
			(DecodedFile::BinaryIndex(binx), _) => binx.show_editor(&mut (), ui),
			(DecodedFile::GenericCPF(cpf), _) => cpf.show_editor(&mut (), ui),
			(DecodedFile::ObjectData(objd), _) => objd.show_editor(&mut (), ui),
			(DecodedFile::SimOutfits(skin), DecodedFileEditorState::SimOutfits(state)) => {
				skin.show_editor(state, ui)
			}
			(
				DecodedFile::ResourceCollection(rcol),
				DecodedFileEditorState::ResourceCollection(state),
			) => rcol.show_editor(state, ui),
			(DecodedFile::TextList(str), DecodedFileEditorState::TextList(state)) => {
				str.show_editor(state, ui)
			}
			(
				DecodedFile::BehaviourFunction(bhav),
				DecodedFileEditorState::BehaviourFunction(state),
			) => bhav.show_editor(state, ui),
			_ => panic!(),
		}
	}
}

pub fn editor_supported(file_type: DBPFFileType) -> bool {
	match file_type {
        DBPFFileType::Known(
            // CPF
            KnownDBPFFileType::TrackSettings |
            KnownDBPFFileType::FloorXML |
            KnownDBPFFileType::NeighbourhoodObjectXML |
            KnownDBPFFileType::WantsXML |
            KnownDBPFFileType::MeshOverlayXML |
            KnownDBPFFileType::BinaryIndex |
            KnownDBPFFileType::FaceModifierXML |
            KnownDBPFFileType::TextureOverlayXML |
            KnownDBPFFileType::FenceXML |
            KnownDBPFFileType::SkinToneXML |
            KnownDBPFFileType::MaterialOverride |
            KnownDBPFFileType::Collection |
            KnownDBPFFileType::FaceNeutralXML |
            KnownDBPFFileType::HairToneXML |
            KnownDBPFFileType::FaceRegionXML |
            KnownDBPFFileType::FaceArchetypeXML |
            KnownDBPFFileType::SimDataXML |
            KnownDBPFFileType::RoofXML |
            KnownDBPFFileType::PetBodyOptions |
            KnownDBPFFileType::WallXML |
            KnownDBPFFileType::PropertySet |
            KnownDBPFFileType::SimDNA |
            KnownDBPFFileType::VersionInformation |
            KnownDBPFFileType::IDReferenceFile |

            // RCOL
            KnownDBPFFileType::TextureResource |
            KnownDBPFFileType::MaterialDefinition |
            KnownDBPFFileType::GeometricDataContainer |

            // STR
            KnownDBPFFileType::TextList |
            KnownDBPFFileType::CatalogDescription |
            KnownDBPFFileType::PieMenuStrings |

            // BHAV
            KnownDBPFFileType::SimanticsBehaviourFunction |

            KnownDBPFFileType::ObjectData
        ) => true,
        _ => false,
    }
}

pub(crate) fn drag_fn<T: Numeric>(name: &str, value: &mut T, ui: &mut Ui) -> Response {
	ui.label(name);
	let res = ui.add(DragValue::new(value).hexadecimal(1, false, false));
	ui.end_row();
	res
}

fn drag_option_fn<T: Numeric>(
	name: &str,
	mut value: &mut Option<T>,
	default: T,
	ui: &mut Ui,
) -> Response {
	ui.label(name);
	let mut has_value = value.is_some();
	let res = ui.horizontal(|ui| {
		let mut res = ui.checkbox(&mut has_value, "");
		match (&mut value, has_value) {
			(Some(v), true) => {
				res |= ui.add(DragValue::new(v).hexadecimal(1, false, false));
			}
			(Some(_), false) => {
				*value = None;
			}
			(None, true) => {
				*value = Some(default);
			}
			(None, false) => {}
		}
		res
	});
	ui.end_row();
	res.response | res.inner
}

fn drag_checkbox_fn<const N: usize, T: Numeric + TryFrom<usize>>(
	name: &str,
	value: &mut T,
	bit_names: [&str; N],
	ui: &mut Ui,
) -> Response
where
	<T as TryFrom<usize>>::Error: Debug,
	usize: TryFrom<T>,
	<usize as TryFrom<T>>::Error: Debug,
{
	ui.label(name);
	let res = ui.with_layout(
		egui::Layout::left_to_right(Align::TOP).with_main_wrap(true),
		|ui| {
			let res = ui.add(DragValue::new(value).hexadecimal(1, false, false));

			let mut value_clone = usize::try_from(*value).unwrap();
			let res = bit_names.iter().enumerate().fold(res, |res, (i, c_name)| {
				let mask = 1 << i;
				let o = (value_clone & mask) > 0;
				let mut c = o;
				let res = res | ui.checkbox(&mut c, *c_name);
				if c != o {
					value_clone = (value_clone & !mask) | (if c { 1 } else { 0 } << i);
				}
				res
			});
			*value = T::try_from(value_clone).unwrap();
			res
		},
	);
	ui.end_row();
	res.response | res.inner
}
