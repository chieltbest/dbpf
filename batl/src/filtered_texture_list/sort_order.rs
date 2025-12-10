// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::texture_finder::FoundTexture;
use dbpf::internal_file::resource_collection::texture_resource::TextureFormat;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, PartialEq};
use std::ops::Not;

pub trait SortOrder {
	fn sort_cmp(&self, tex1: &FoundTexture, tex2: &FoundTexture) -> Ordering;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Sequence, Serialize, Deserialize)]
pub enum TextureSortType {
	Path,
	Type,
	Group,
	Instance,
	Width,
	Height,
	Format,
	MipLevels,
	MemorySize,
}

impl TextureSortType {
	pub fn default_direction(&self) -> SortDirection {
		match self {
			TextureSortType::Path
			| TextureSortType::Type
			| TextureSortType::Group
			| TextureSortType::Instance => SortDirection::Ascending,
			_ => SortDirection::Descending,
		}
	}
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SortDirection {
	Ascending,
	Descending,
}

impl Not for SortDirection {
	type Output = SortDirection;

	fn not(self) -> Self::Output {
		match self {
			SortDirection::Ascending => SortDirection::Descending,
			SortDirection::Descending => SortDirection::Ascending,
		}
	}
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TextureSortOperation {
	pub sort_type: TextureSortType,
	pub direction: SortDirection,
}

fn texture_format_user_order(texture_format: TextureFormat) -> usize {
	match texture_format {
		// 4 bits per pixel
		TextureFormat::DXT1 => 0,
		// 8 bits per pixel
		TextureFormat::DXT3 => 1,
		TextureFormat::DXT5 => 2,
		TextureFormat::Grayscale => 3,
		TextureFormat::Alpha => 4,
		// 24 bits per pixel
		TextureFormat::RawRGB24 => 5,
		// 32 bits per pixel
		TextureFormat::RawARGB32 => 6,
		// 'alt' modes
		TextureFormat::AltRGB24 => 7,
		TextureFormat::AltARGB32 => 8,
	}
}

impl SortOrder for TextureSortOperation {
	fn sort_cmp(&self, tex1: &FoundTexture, tex2: &FoundTexture) -> Ordering {
		let order = match self.sort_type {
			TextureSortType::Path => tex1.id.path.cmp(&tex2.id.path),
			TextureSortType::Type => tex1.id.tgi.type_id.code().cmp(&tex2.id.tgi.type_id.code()),
			TextureSortType::Group => tex1.id.tgi.group_id.cmp(&tex2.id.tgi.group_id),
			TextureSortType::Instance => tex1.id.tgi.instance_id.cmp(&tex2.id.tgi.instance_id),
			TextureSortType::Width => tex1.width.cmp(&tex2.width),
			TextureSortType::Height => tex1.height.cmp(&tex2.height),
			TextureSortType::Format => {
				texture_format_user_order(tex1.format).cmp(&texture_format_user_order(tex2.format))
			}
			TextureSortType::MipLevels => tex1.mip_levels.cmp(&tex2.mip_levels),
			TextureSortType::MemorySize => tex1.memory_size.cmp(&tex2.memory_size),
		};

		match self.direction {
			SortDirection::Ascending => order,
			SortDirection::Descending => order.reverse(),
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureSorter {
	operations: Vec<TextureSortOperation>,
}

impl TextureSorter {
	pub fn set_sort(&mut self, operation: TextureSortOperation) {
		if let Some((idx, _)) = self
			.operations
			.iter()
			.enumerate()
			.find(|(_i, op)| op.sort_type == operation.sort_type)
		{
			// move the found operation to the front
			self.operations[0..=idx].rotate_right(1);
			self.operations[0] = operation;
		} else {
			self.operations.insert(0, operation);
		}
	}

	pub fn get_sort(&self) -> &TextureSortOperation {
		&self.operations[0]
	}
}

impl SortOrder for TextureSorter {
	fn sort_cmp(&self, tex1: &FoundTexture, tex2: &FoundTexture) -> Ordering {
		self.operations.iter().fold(Ordering::Equal, |b, x| {
			b.then_with(|| x.sort_cmp(tex1, tex2))
		})
	}
}

impl Default for TextureSorter {
	fn default() -> Self {
		Self {
			operations: enum_iterator::all()
				.map(|sort_type: TextureSortType| TextureSortOperation {
					direction: sort_type.default_direction(),
					sort_type,
				})
				.collect(),
		}
	}
}
