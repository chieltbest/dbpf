// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::texture_finder::{deser_texture_format, ser_texture_format, FoundTexture};
use dbpf::internal_file::resource_collection::texture_resource::TextureFormat;
use dbpf_utils::editor::vector::VecEditorState;
use dbpf_utils::editor::Editor;
use eframe::egui::{ComboBox, Context, DragValue, Response, Ui};
use eframe::glow;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

pub(crate) trait TextureFilterRule {
	/// filter the texture according to this rule, returns true if the texture should be shown
	fn filter(&self, tex: &FoundTexture) -> bool;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum ComparisonType {
	Less,
	LessEq,
	Greater,
	GreaterEq,
	Equal,
	NotEqual,
}

impl Display for ComparisonType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			ComparisonType::Less => "<",
			ComparisonType::LessEq => "≤",
			ComparisonType::Greater => ">",
			ComparisonType::GreaterEq => "≥",
			ComparisonType::Equal => "=",
			ComparisonType::NotEqual => "≠",
		})
	}
}

impl ComparisonType {
	pub fn check(&self, x: usize, y: usize) -> bool {
		match self {
			ComparisonType::Less => x < y,
			ComparisonType::LessEq => x <= y,
			ComparisonType::Greater => x > y,
			ComparisonType::GreaterEq => x >= y,
			ComparisonType::Equal => x == y,
			ComparisonType::NotEqual => x != y,
		}
	}
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct SerTexFormat(TextureFormat);

impl Serialize for SerTexFormat {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		ser_texture_format(&self.0, serializer)
	}
}

impl<'de> Deserialize<'de> for SerTexFormat {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deser_texture_format(deserializer).map(|t| t.into())
	}
}

impl From<TextureFormat> for SerTexFormat {
	fn from(value: TextureFormat) -> Self {
		Self(value)
	}
}

impl From<SerTexFormat> for TextureFormat {
	fn from(value: SerTexFormat) -> Self {
		value.0
	}
}

fn ser_format_filter<S: Serializer>(
	t: &BTreeSet<TextureFormat>,
	ser: S,
) -> Result<S::Ok, S::Error> {
	let ser_btree: BTreeSet<SerTexFormat> = t.iter().map(|t| (*t).into()).collect();

	ser_btree.serialize(ser)
}

fn deser_format_filter<'a, D>(d: D) -> Result<BTreeSet<TextureFormat>, D::Error>
where
	D: Deserializer<'a>,
{
	Ok(BTreeSet::<SerTexFormat>::deserialize(d)?
		.iter()
		.map(|t| t.0)
		.collect())
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum TextureFilterOperation {
	Width(ComparisonType, usize),
	Height(ComparisonType, usize),
	Memory(ComparisonType, usize),
	#[serde(
		serialize_with = "ser_format_filter",
		deserialize_with = "deser_format_filter"
	)]
	Format(BTreeSet<TextureFormat>),
	Mip(ComparisonType, usize),
}

impl TextureFilterRule for TextureFilterOperation {
	fn filter(&self, tex: &FoundTexture) -> bool {
		let (comp, y) = match self {
			TextureFilterOperation::Format(checked) => {
				return checked.contains(&tex.format);
			}
			TextureFilterOperation::Width(c, x)
			| TextureFilterOperation::Height(c, x)
			| TextureFilterOperation::Memory(c, x)
			| TextureFilterOperation::Mip(c, x) => (c, *x),
		};
		let x = match self {
			TextureFilterOperation::Width(_, _) => tex.width as usize,
			TextureFilterOperation::Height(_, _) => tex.height as usize,
			TextureFilterOperation::Memory(_, _) => tex.memory_size,
			TextureFilterOperation::Mip(_, _) => tex.mip_levels as usize,
			_ => 0,
		};
		comp.check(x, y)
	}
}

impl Default for TextureFilterOperation {
	fn default() -> Self {
		Self::Width(ComparisonType::Greater, 512)
	}
}

impl From<TextureFilterOperation> for usize {
	fn from(value: TextureFilterOperation) -> Self {
		match value {
			TextureFilterOperation::Width(_, _) => 0,
			TextureFilterOperation::Height(_, _) => 1,
			TextureFilterOperation::Memory(_, _) => 2,
			TextureFilterOperation::Format(_) => 3,
			TextureFilterOperation::Mip(_, _) => 4,
		}
	}
}

impl Editor for TextureFilterOperation {
	type EditorState = ();

	fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let mut selected: usize = self.clone().into();

		let ires = ComboBox::from_id_salt(ui.id().with(0))
			.selected_text(match selected {
				0 => "Width",
				1 => "Height",
				2 => "Memory",
				3 => "Format",
				_ => "Mip levels",
			})
			.show_ui(ui, |ui| {
				[
					ui.selectable_value(&mut selected, 0, "Width"),
					ui.selectable_value(&mut selected, 1, "Height"),
					ui.selectable_value(&mut selected, 2, "Memory"),
					ui.selectable_value(&mut selected, 3, "Format"),
					ui.selectable_value(&mut selected, 4, "Mip levels"),
				]
				.into_iter()
				.reduce(|r1, r2| r1.union(r2))
				.unwrap()
			});

		if ires.inner.as_ref().is_some_and(|r| r.changed()) {
			match selected {
				0 => *self = Self::Width(ComparisonType::Greater, 512),
				1 => *self = Self::Height(ComparisonType::Greater, 512),
				2 => *self = Self::Memory(ComparisonType::Greater, 1_000_000),
				3 => {
					*self = Self::Format(BTreeSet::from([
						TextureFormat::RawRGB24,
						TextureFormat::RawARGB32,
						TextureFormat::AltRGB24,
						TextureFormat::AltARGB32,
					]))
				}
				_ => *self = Self::Mip(ComparisonType::Greater, 1),
			}
		}

		let mut res = ires.response;
		if let Some(inner) = ires.inner {
			res |= inner;
		}

		res |= match self {
			TextureFilterOperation::Width(c, y)
			| TextureFilterOperation::Height(c, y)
			| TextureFilterOperation::Memory(c, y)
			| TextureFilterOperation::Mip(c, y) => {
				let res = ComboBox::from_id_salt(ui.id().with(1))
					.width(40.0)
					.selected_text(format!("{c}"))
					.show_ui(ui, |ui| {
						[
							ComparisonType::Less,
							ComparisonType::LessEq,
							ComparisonType::Greater,
							ComparisonType::GreaterEq,
							ComparisonType::Equal,
							ComparisonType::NotEqual,
						]
						.into_iter()
						.map(|ct| ui.selectable_value(c, ct, format!("{ct}")))
						.reduce(|r1, r2| r1 | r2)
						.unwrap()
					});
				let mut res = if let Some(inner) = res.inner {
					res.response | inner
				} else {
					res.response
				};
				res |= ui.add(DragValue::new(y));
				res
			}
			TextureFilterOperation::Format(checked) => {
				let mut inner = None;
				let res = ui.menu_button("choose", |ui| {
					let res = [
						TextureFormat::Alpha,
						TextureFormat::Grayscale,
						TextureFormat::RawRGB24,
						TextureFormat::RawARGB32,
						TextureFormat::AltRGB24,
						TextureFormat::AltARGB32,
						TextureFormat::DXT1,
						TextureFormat::DXT3,
						TextureFormat::DXT5,
					]
					.iter()
					.map(|tf| {
						let mut new = checked.contains(tf);
						let res = ui.checkbox(&mut new, format!("{tf:?}"));
						if res.changed() {
							if new {
								checked.insert(*tf);
							} else {
								checked.remove(tf);
							}
						}
						res
					})
					.reduce(|r1, r2| r1 | r2)
					.unwrap();
					inner = Some(res);
				});

				if let Some(inner) = inner {
					res.response | inner
				} else {
					res.response
				}
			}
		};

		res
	}
}

#[derive(Clone, Hash, Default, Debug, Serialize, Deserialize)]
pub struct TextureFilter {
	pub operations: Vec<TextureFilterOperation>,
}

impl TextureFilterRule for TextureFilter {
	fn filter(&self, tex: &FoundTexture) -> bool {
		!self.operations.iter().any(|filter| !filter.filter(tex))
	}
}

impl Editor for TextureFilter {
	type EditorState = VecEditorState<TextureFilterOperation>;

	fn new_editor(&self, context: &Context, gl: &Option<Arc<glow::Context>>) -> Self::EditorState {
		self.operations.new_editor(context, gl)
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		self.operations.show_editor(state, ui)
	}
}
