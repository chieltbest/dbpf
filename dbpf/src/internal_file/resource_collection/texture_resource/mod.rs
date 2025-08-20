// SPDX-FileCopyrightText: 2023-2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod decoded_texture;

use std::{
	cmp::max,
	fmt::Debug,
	io::{Cursor, Read, Write},
};

use binrw::{args, binrw, BinResult, Error};
use ddsfile::{D3DFormat, Dds, DxgiFormat, NewD3dParams, PixelFormatFlags};
use decoded_texture::DecodedTexture;
use enum_iterator::Sequence;
use log::error;
use thiserror::Error;

use crate::{
	common::BigString,
	internal_file::resource_collection::{
		texture_resource::decoded_texture::{ShrinkDirection, ShrinkResult},
		FileName, ResourceBlockVersion,
	},
};

const TEXPRESSO_PARAMS: texpresso::Params = texpresso::Params {
	algorithm: texpresso::Algorithm::IterativeClusterFit,
	weights: texpresso::COLOUR_WEIGHTS_PERCEPTUAL,
	weigh_colour_by_alpha: true,
};

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TextureFormat {
	RawARGB32 = 1,
	RawRGB24 = 2,
	Alpha = 3,
	DXT1 = 4,
	#[default]
	DXT3 = 5,
	Grayscale = 6,
	AltARGB32 = 7,
	DXT5 = 8,
	AltRGB24 = 9,
}

impl TextureFormat {
	/// get the size that a texture will be in memory when encoded in this texture format
	pub fn compressed_size(&self, width: usize, height: usize) -> usize {
		match self {
			TextureFormat::RawARGB32 | TextureFormat::AltARGB32 => width * height * 4,
			TextureFormat::RawRGB24 | TextureFormat::AltRGB24 => width * height * 3,
			TextureFormat::Grayscale | TextureFormat::Alpha => width * height,
			TextureFormat::DXT1 => texpresso::Format::Bc1.compressed_size(width, height),
			TextureFormat::DXT3 => texpresso::Format::Bc2.compressed_size(width, height),
			TextureFormat::DXT5 => texpresso::Format::Bc3.compressed_size(width, height),
		}
	}

	pub fn decompress(&self, data: &[u8], width: usize, height: usize, output: &mut [u8]) {
		match self {
			TextureFormat::RawARGB32 | TextureFormat::AltARGB32 => {
				output.copy_from_slice(
					data.chunks_exact(4)
						.flat_map(|px| [px[2], px[1], px[0], px[3]])
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::RawRGB24 | TextureFormat::AltRGB24 => {
				output.copy_from_slice(
					data.chunks_exact(3)
						.flat_map(|px| [px[2], px[1], px[0], 0xFF])
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::Grayscale => {
				output.copy_from_slice(
					data.iter()
						.flat_map(|px| [*px, *px, *px, 0xFF])
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::Alpha => {
				output.copy_from_slice(
					data.iter()
						.flat_map(|px| [0, 0, 0, *px])
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::DXT1 => texpresso::Format::Bc1.decompress(data, width, height, output),
			TextureFormat::DXT3 => texpresso::Format::Bc2.decompress(data, width, height, output),
			TextureFormat::DXT5 => texpresso::Format::Bc3.decompress(data, width, height, output),
		}
	}

	pub fn compress(&self, data: &[u8], width: usize, height: usize, output: &mut [u8]) {
		match self {
			TextureFormat::RawARGB32 | TextureFormat::AltARGB32 => {
				output.copy_from_slice(
					data.chunks_exact(4)
						.flat_map(|px| [px[2], px[1], px[0], px[3]])
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::RawRGB24 | TextureFormat::AltRGB24 => {
				output.copy_from_slice(
					data.chunks_exact(4)
						.flat_map(|px| [px[2], px[1], px[0]])
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::Grayscale => {
				output.copy_from_slice(
					data.chunks_exact(4)
						.map(|px| (((px[0] as u16) + (px[1] as u16) + (px[2] as u16)) / 3) as u8)
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::Alpha => {
				output.copy_from_slice(
					data.chunks_exact(4)
						.map(|px| px[3])
						.collect::<Vec<u8>>()
						.as_slice(),
				);
			}
			TextureFormat::DXT1 => {
				texpresso::Format::Bc1.compress(data, width, height, TEXPRESSO_PARAMS, output)
			}
			TextureFormat::DXT3 => {
				texpresso::Format::Bc2.compress(data, width, height, TEXPRESSO_PARAMS, output)
			}
			TextureFormat::DXT5 => {
				texpresso::Format::Bc3.compress(data, width, height, TEXPRESSO_PARAMS, output)
			}
		}
	}
}

#[binrw]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmbeddedTextureResourceMipLevel {
	#[br(temp)]
	#[bw(calc = data.len() as u32)]
	num_input_bytes: u32,
	#[br(args{ count: num_input_bytes as usize })]
	pub data: Vec<u8>,
}

impl EmbeddedTextureResourceMipLevel {
	pub fn decompress(
		&self,
		width: usize,
		height: usize,
		format: TextureFormat,
	) -> BinResult<Vec<u8>> {
		let mut out = vec![0; width * height * 4];
		// move this logic to binread?
		let compressed_size = format.compressed_size(width, height);
		if compressed_size != self.data.len() {
			error!(
				"decompress failed: {:?} {} {compressed_size} {} {}",
				format,
				self.data.len(),
				width,
				height
			);
			return Err(Error::AssertFail {
				pos: 0,
				message: format!(
					"Mipmap level calculated size {compressed_size} \
                 does not match data length {} \
                 with texture size {width}x{height}",
					self.data.len()
				),
			});
		}
		format.decompress(&self.data, width, height, &mut out);
		Ok(out)
	}

	pub fn compress(&mut self, width: usize, height: usize, format: TextureFormat, data: &[u8]) {
		self.data.resize(format.compressed_size(width, height), 0);
		format.compress(data, width, height, &mut self.data);
	}

	pub fn new(width: usize, height: usize, format: TextureFormat, data: &[u8]) -> Self {
		let mut new = Self { data: vec![] };
		new.compress(width, height, format, data);
		new
	}
}

// impl Debug for EmbeddedTextureResourceMipLevel {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let mut out_strings = vec![String::new(); 4];
//         for channel in 0..4 {
//             for y in 0..self.height {
//                 for x in 0..self.width {
//                     out_strings[channel].push_str(
//                         match self.data[((y * self.width + x) * 4 + channel as u32) as usize] {
//                             0..=15 => " ",
//                             16..=31 => ".",
//                             32..=47 => "-",
//                             48..=63 => "+",
//                             64..=79 => "x",
//                             80..=95 => "X",
//                             96..=111 => "#",
//                             _ => "@",
//                         })
//                 }
//                 out_strings[channel].push_str("\n");
//             }
//         }
//         struct DisplayString<'a> {
//             str: &'a String,
//         }
//         impl<'a> Debug for DisplayString<'a> {
//             fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//                 writeln!(f, "")?;
//                 write!(f, "{}", self.str)
//             }
//         }
//         f.debug_struct("EmbeddedTextureResourceMipLevel")
//             .field("width", &self.width)
//             .field("height", &self.height)
//             .field("red", &DisplayString { str: &out_strings[0] })
//             .field("green", &DisplayString { str: &out_strings[1] })
//             .field("blue", &DisplayString { str: &out_strings[2] })
//             .field("alpha", &DisplayString { str: &out_strings[3] })
//             .finish()
//     }
// }

#[binrw]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureResourceData {
	#[brw(magic = 0u8)]
	Embedded(EmbeddedTextureResourceMipLevel),
	#[brw(magic = 1u8)]
	LIFOFile { file_name: BigString },
}

#[binrw]
#[brw(magic = 10f32)]
#[derive(Default)]
struct FormatFlag;

#[binrw]
#[brw(import{version: ResourceBlockVersion, mip_levels: u32})]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextureResourceTexture {
	#[br(if (matches ! (version, ResourceBlockVersion::V9), mip_levels))]
	#[bw(if (matches ! (version, ResourceBlockVersion::V9)), calc = entries.len() as u32)]
	pub mip_levels: u32,

	#[br(count = mip_levels)]
	#[bw(assert(matches ! (version, ResourceBlockVersion::V9) || entries.len() as u32 == mip_levels,
        "Cannot have differing amount of mip levels while resource version is not V9"))]
	pub entries: Vec<TextureResourceData>,

	// 0xFF000000 or 0xFFFFFFFF when not uploaded online
	pub creator_id: u32,
	#[brw(if (matches ! (version, ResourceBlockVersion::V9)))]
	#[br(temp)]
	#[bw(calc = FormatFlag)]
	format_flag: FormatFlag,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Sequence)]
pub enum KnownPurpose {
	#[default]
	#[brw(magic = 1.0f32)]
	Object,
	#[brw(magic = 2.0f32)]
	Outfit,
	#[brw(magic = 3.0f32)]
	Interface,
}

impl From<KnownPurpose> for f32 {
	fn from(value: KnownPurpose) -> Self {
		match value {
			KnownPurpose::Object => 1.0,
			KnownPurpose::Outfit => 2.0,
			KnownPurpose::Interface => 3.0,
		}
	}
}

#[binrw]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Purpose {
	Known(KnownPurpose),
	Unknown(f32),
}

impl Default for Purpose {
	fn default() -> Self {
		Self::Known(KnownPurpose::default())
	}
}

#[binrw]
#[brw(import{version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureResource {
	pub file_name: FileName,

	pub width: u32,
	pub height: u32,

	format: TextureFormat,
	#[br(temp)]
	#[bw(calc = self.mip_levels() as u32)]
	mip_levels: u32,
	pub purpose: Purpose,
	#[br(temp)]
	#[bw(calc = textures.len() as u32)]
	num_textures: u32,
	pub unknown: u32,

	#[brw(if (matches ! (version, ResourceBlockVersion::V9)))]
	pub file_name_repeat: BigString,

	#[br(args{count: num_textures as usize, inner: args ! {version, mip_levels}})]
	#[bw(args{version, mip_levels: mip_levels})]
	pub textures: Vec<TextureResourceTexture>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum DdsFormat {
	D3DFormat(D3DFormat),
	DxgiFormat(DxgiFormat),
}

#[derive(Debug, Error)]
pub enum DdsError {
	#[error(transparent)]
	DdsFile(#[from] ddsfile::Error),
	#[error("Unsupported format {0:?}")]
	UnsupportedFormat(Option<DdsFormat>),
	#[error(transparent)]
	IoError(#[from] std::io::Error),
	#[error("Tried to export a texture with lifo mipmaps")]
	LifoExport,
}

impl TextureResource {
	/// guess the best matching size for the given mipmap level
	pub fn mip_size(&self, mip_level: usize) -> (usize, usize) {
		let width = max(self.width >> mip_level, 1) as usize;
		let height = max(self.height >> mip_level, 1) as usize;
		(width, height)
	}

	fn dds_to_txtr_format(dds: &Dds) -> Result<(TextureFormat, fn(Vec<u8>) -> Vec<u8>), DdsError> {
		let identity = |data| data;
		match dds.get_d3d_format() {
			Some(format) => Ok(match format {
				D3DFormat::A8B8G8R8 => (TextureFormat::RawARGB32, |data: Vec<u8>| {
					data.chunks_exact(4)
						.flat_map(|c| [c[2], c[1], c[0], c[3]])
						.collect()
				}),
				D3DFormat::A8R8G8B8 => (TextureFormat::RawARGB32, identity),
				D3DFormat::R8G8B8 => (TextureFormat::RawRGB24, identity),
				D3DFormat::X8B8G8R8 => (TextureFormat::RawARGB32, |data: Vec<u8>| {
					data.chunks_exact(4)
						.flat_map(|c| [c[2], c[1], c[0], 0xFF])
						.collect()
				}),
				D3DFormat::X8R8G8B8 => (TextureFormat::RawARGB32, |data: Vec<u8>| {
					data.chunks_exact(4)
						.flat_map(|c| [c[0], c[1], c[2], 0xFF])
						.collect()
				}),
				D3DFormat::A8 => (TextureFormat::Alpha, identity),
				D3DFormat::L8 => (TextureFormat::Grayscale, identity),
				D3DFormat::DXT1 => (TextureFormat::DXT1, identity),
				D3DFormat::DXT3 => (TextureFormat::DXT3, identity),
				D3DFormat::DXT5 => (TextureFormat::DXT5, identity),
				_ => Err(DdsError::UnsupportedFormat(Some(DdsFormat::D3DFormat(
					format,
				))))?,
			}),
			_ => match dds.get_dxgi_format() {
				Some(format) => Ok(match format {
					DxgiFormat::R8G8B8A8_Typeless
					| DxgiFormat::R8G8B8A8_UNorm
					| DxgiFormat::R8G8B8A8_UNorm_sRGB
					| DxgiFormat::R8G8B8A8_UInt
					| DxgiFormat::R8G8B8A8_SNorm
					| DxgiFormat::R8G8B8A8_SInt => (TextureFormat::RawARGB32, |data: Vec<u8>| {
						data.chunks_exact(4)
							.flat_map(|c| [c[1], c[2], c[3], c[0]])
							.collect()
					}),
					DxgiFormat::R8_Typeless
					| DxgiFormat::R8_UNorm
					| DxgiFormat::R8_UInt
					| DxgiFormat::R8_SNorm
					| DxgiFormat::R8_SInt => (TextureFormat::Grayscale, identity),
					DxgiFormat::A8_UNorm => (TextureFormat::Alpha, identity),
					DxgiFormat::BC1_Typeless
					| DxgiFormat::BC1_UNorm
					| DxgiFormat::BC1_UNorm_sRGB => (TextureFormat::DXT1, identity),
					DxgiFormat::BC2_Typeless
					| DxgiFormat::BC2_UNorm
					| DxgiFormat::BC2_UNorm_sRGB => (TextureFormat::DXT3, identity),
					DxgiFormat::BC3_Typeless
					| DxgiFormat::BC3_UNorm
					| DxgiFormat::BC3_UNorm_sRGB => (TextureFormat::DXT5, identity),
					DxgiFormat::B8G8R8A8_Typeless
					| DxgiFormat::B8G8R8A8_UNorm
					| DxgiFormat::B8G8R8A8_UNorm_sRGB => (TextureFormat::RawARGB32, identity),
					DxgiFormat::B8G8R8X8_Typeless
					| DxgiFormat::B8G8R8X8_UNorm
					| DxgiFormat::B8G8R8X8_UNorm_sRGB => (TextureFormat::AltARGB32, identity),
					_ => Err(DdsError::UnsupportedFormat(Some(DdsFormat::DxgiFormat(
						format,
					))))?,
				}),
				_ => match dds.header.spf.flags {
					PixelFormatFlags::ALPHA if dds.header.spf.rgb_bit_count.unwrap_or(8) == 8 => {
						Ok((TextureFormat::Alpha, identity))
					}
					PixelFormatFlags::LUMINANCE
						if dds.header.spf.rgb_bit_count.unwrap_or(8) == 8 =>
					{
						Ok((TextureFormat::Grayscale, identity))
					}
					_ => Err(DdsError::UnsupportedFormat(None)),
				},
			},
		}
	}

	pub fn import_dds<R: Read>(reader: R) -> Result<Self, DdsError> {
		let dds = Dds::read(reader)?;
		let width = dds.header.width;
		let height = dds.header.height;

		let (format, post_process_fn) = Self::dds_to_txtr_format(&dds)?;

		let mut texture = Self {
			width,
			height,
			format,
			unknown: 0,
			textures: vec![],
			..Default::default()
		};

		for i in 0..dds.header.depth.unwrap_or(1) {
			let mut cur = Cursor::new(dds.get_data(i)?);
			let mut entries = vec![];

			for mip_i in 0..dds.header.mip_map_count.unwrap_or(1) {
				let (width, height) = texture.mip_size(mip_i as usize);
				let data_size = format.compressed_size(width, height);
				let mut data = vec![0; data_size];
				cur.read_exact(&mut data)?;

				entries.insert(
					0,
					TextureResourceData::Embedded(EmbeddedTextureResourceMipLevel {
						data: post_process_fn(data),
					}),
				)
			}

			texture.textures.push(TextureResourceTexture {
				entries,
				creator_id: 0xFFFFFFFF,
			});
		}

		Ok(texture)
	}

	fn txtr_format_to_dds(texture_format: TextureFormat) -> (D3DFormat, Option<PixelFormatFlags>) {
		match texture_format {
			TextureFormat::RawARGB32 => (D3DFormat::A8R8G8B8, None),
			TextureFormat::RawRGB24 => (D3DFormat::R8G8B8, None),
			TextureFormat::Alpha => (D3DFormat::A8, Some(PixelFormatFlags::ALPHA)),
			TextureFormat::DXT1 => (D3DFormat::DXT1, None),
			TextureFormat::DXT3 => (D3DFormat::DXT3, None),
			TextureFormat::Grayscale => (D3DFormat::L8, Some(PixelFormatFlags::LUMINANCE)),
			TextureFormat::AltARGB32 => (D3DFormat::A8R8G8B8, None),
			TextureFormat::DXT5 => (D3DFormat::DXT5, None),
			TextureFormat::AltRGB24 => (D3DFormat::R8G8B8, None),
		}
	}

	pub fn export_dds<W: Write>(&self, writer: &mut W) -> Result<(), DdsError> {
		let (format, pixelformat) = Self::txtr_format_to_dds(self.format);

		let mut dds = Dds::new_d3d(NewD3dParams {
			height: self.height,
			width: self.width,
			depth: Some(self.textures.len() as u32),
			format,
			mipmap_levels: Some(self.mip_levels() as u32),
			caps2: None,
		})?;

		if let Some(pxformat) = pixelformat {
			dds.header.spf.flags = pxformat;
		}

		for (i, texture) in self.textures.iter().enumerate() {
			let mut dds_data = Cursor::new(dds.get_mut_data(i as u32)?);
			for mip in texture.entries.iter().rev() {
				match mip {
					TextureResourceData::Embedded(e) => {
						dds_data.write_all(&e.data)?;
					}
					TextureResourceData::LIFOFile { .. } => return Err(DdsError::LifoExport),
				}
			}
		}

		dds.write(writer)?;
		Ok(())
	}

	/// decompress a single texture given by its texture and mip index
	/// the mip index goes from 0 (largest) to mip_levels (smallest)
	/// if a mip level is represented as a LIFOStream then this function will return None
	pub fn decompress(&self, texture_index: usize, mip_index: usize) -> BinResult<DecodedTexture> {
		let TextureResourceData::Embedded(ref data) =
			self.textures[texture_index].entries[mip_index]
		else {
			return Err(Error::AssertFail {
				pos: 0,
				message: "Mipmap entry is a LIFO file".to_string(),
			});
		};
		let mip_shift = (self.textures[texture_index].entries.len() - 1) - mip_index;
		let (width, height) = self.mip_size(mip_shift);
		let data = data.decompress(width, height, self.format)?;
		Ok(DecodedTexture {
			data,
			width,
			height,
		})
	}

	pub fn decompress_all(&self) -> Vec<Vec<BinResult<DecodedTexture>>> {
		(0..self.textures.len())
			.map(|texture_index| {
				(0..self.textures[texture_index].entries.len())
					.map(|mip_index| self.decompress(texture_index, mip_index))
					.collect()
			})
			.collect()
	}

	pub fn compress_replace(
		&mut self,
		decoded_texture: DecodedTexture,
		update_format: Option<TextureFormat>,
	) {
		self.width = decoded_texture.width as u32;
		self.height = decoded_texture.height as u32;
		self.textures.truncate(1);
		if let Some(format) = update_format {
			self.format = format;
		}
		if let Some(tex) = self.textures.first_mut() {
			tex.entries = vec![TextureResourceData::Embedded(
				EmbeddedTextureResourceMipLevel::new(
					decoded_texture.width,
					decoded_texture.height,
					self.format,
					&decoded_texture.data,
				),
			)];
		}
	}

	/// recompress all textures in the new format and update the image data
	///
	/// creates a copy of the texture; if the conversion fails the original data is unaffected
	pub fn recompress_with_format(&self, texture_format: TextureFormat) -> BinResult<Self> {
		let previous_format = self.format;

		let mut new = self.clone();

		if new.format != texture_format {
			new.format = texture_format;
			new.textures.iter_mut().try_for_each(|texture| {
				let total_mip_levels = texture.entries.len();
				texture.entries.iter_mut().enumerate().try_for_each(
					|(mip_level, entry)| -> BinResult<()> {
						match entry {
							TextureResourceData::Embedded(mip) => {
								let mip_shift = (total_mip_levels - 1) - mip_level;
								let (width, height) = self.mip_size(mip_shift);
								let texture_data =
									mip.decompress(width, height, previous_format)?;
								mip.compress(width, height, texture_format, &texture_data);
								Ok(())
							}
							TextureResourceData::LIFOFile { .. } => Ok(()),
						}
					},
				)
			})?;
		}

		Ok(new)
	}

	pub fn get_format(&self) -> TextureFormat {
		self.format
	}

	/// remove n mipmap levels, starting from the largest first, effectively decreasing the size of the texture
	/// the number of mipmap levels will be reduced by mip_levels
	pub fn remove_largest_mip_levels(&mut self, mip_levels: usize) {
		let (width, height) = self.mip_size(mip_levels);
		self.width = width as u32;
		self.height = height as u32;
		self.textures.iter_mut().for_each(|texture| {
			texture.entries.truncate(texture.entries.len() - mip_levels);
		});
	}

	/// remove n mipmap levels, starting from the smallest first
	/// mip_levels is the number of mipmap levels that should be removed
	pub fn remove_smallest_mip_levels(&mut self, mip_levels: usize) {
		self.textures.iter_mut().for_each(|texture| {
			texture.entries.drain(..mip_levels);
		});
	}

	/// remove all but the largest mipmap level
	pub fn remove_smaller_mip_levels(&mut self) {
		let cur_mip_levels = self.mip_levels();
		self.remove_smallest_mip_levels(cur_mip_levels - 1);
	}

	/// attempt to do a certain number of mipmap additions in one go, returns the actual amount added
	pub fn add_extra_mip_levels(&mut self, mip_levels: usize, preserve_alpha: Option<u8>) -> usize {
		assert!(mip_levels >= 1);
		let cur_mip_levels = self.mip_levels();
		let (smallest_width, smallest_height) = self.mip_size(cur_mip_levels - 1);
		let cur_smallest_textures = self
			.textures
			.iter()
			.map(|tex| {
				let smallest_mipmap = tex.entries.first().cloned();
				match smallest_mipmap {
					Some(TextureResourceData::Embedded(e)) => Ok(DecodedTexture {
						data: e
							.decompress(smallest_width, smallest_height, self.format)
							.map_err(|_| ())?,
						width: smallest_width,
						height: smallest_height,
					}),
					_ => Err(()),
				}
			})
			.collect::<Result<Vec<_>, _>>();
		let Ok(mut cur_textures) = cur_smallest_textures else {
			return 0;
		};

		for i in 0..mip_levels {
			if cur_textures
				.iter_mut()
				.try_for_each(|tex| {
					if matches!(
						tex.shrink(preserve_alpha, ShrinkDirection::Both),
						ShrinkResult::Ok
					) {
						Ok(())
					} else {
						Err(())
					}
				})
				.is_err()
			{
				return i;
			}
			self.textures
				.iter_mut()
				.zip(cur_textures.iter())
				.for_each(|(tex, new)| {
					tex.entries.insert(
						0,
						TextureResourceData::Embedded(EmbeddedTextureResourceMipLevel::new(
							new.width,
							new.height,
							self.format,
							&new.data,
						)),
					);
				});
		}
		mip_levels
	}

	pub fn add_max_mip_levels(&mut self, preserve_alpha: Option<u8>) {
		let max = self.max_mip_levels();
		let cur = self.mip_levels();
		let extra = max - cur;
		if extra > 0 {
			self.add_extra_mip_levels(extra, preserve_alpha);
		}
	}

	pub fn mip_levels(&self) -> usize {
		self.textures.first().map(|t| t.entries.len()).unwrap_or(0)
	}

	pub fn max_mip_levels(&self) -> usize {
		let mut width = self.width;
		let mut height = self.height;
		if width == 0 || height == 0 {
			return 0;
		}
		let mut i = 1;
		while (width > 1) || (height > 1) {
			if DecodedTexture::can_shrink_dimensions(
				width as usize,
				height as usize,
				ShrinkDirection::Both,
			) == ShrinkResult::Unable
			{
				return 1;
			}
			i += 1;
			width = max(width >> 1, 1);
			height = max(height >> 1, 1);
		}
		i
	}

	fn any_lifo(&self) -> bool {
		self.textures.iter().any(|t| {
			t.entries
				.iter()
				.any(|e| matches!(e, TextureResourceData::LIFOFile { .. }))
		})
	}

	pub fn can_shrink(&self, shrink_direction: ShrinkDirection) -> bool {
		let mut results = self.textures.iter().map(|t| {
			let (mip_width, mip_height) = self.mip_size(t.entries.len());
			DecodedTexture::can_shrink_dimensions(mip_width, mip_height, shrink_direction)
		});
		!results.any(|r| r == ShrinkResult::Unable)
			&& DecodedTexture::can_shrink_dimensions(
				self.width as usize,
				self.height as usize,
				shrink_direction,
			) != ShrinkResult::Small
			&& !self.any_lifo()
	}

	pub fn shrink(
		&mut self,
		preserve_alpha: Option<u8>,
		shrink_direction: ShrinkDirection,
	) -> BinResult<bool> {
		if !self.can_shrink(shrink_direction) {
			return Ok(false);
		}

		let format = self.format;
		for texture in 0..self.textures.len() {
			let num_mip = self.textures[texture].entries.len();
			for i in 0..num_mip {
				let mip_level = num_mip - i - 1;
				let (mip_width, mip_height) = self.mip_size(mip_level);
				let texture = &mut self.textures[texture];
				if let TextureResourceData::Embedded(mip) = &mut texture.entries[i] {
					let mut decoded = DecodedTexture {
						data: mip.decompress(mip_width, mip_height, format)?,
						width: mip_width,
						height: mip_height,
					};
					decoded.shrink(preserve_alpha, shrink_direction);
					mip.compress(decoded.width, decoded.height, format, &decoded.data);
				}
			}
		}

		let old_width = self.width;
		let old_height = self.height;
		if shrink_direction != ShrinkDirection::Vertical {
			self.width /= 2;
		}
		if shrink_direction != ShrinkDirection::Horizontal {
			self.height /= 2;
		}

		let mut mip_removed = false;
		let max_mip = self.max_mip_levels();
		for texture in &mut self.textures {
			if texture.entries.len() > max_mip {
				texture.entries.remove(0);
				mip_removed = true;
			}
		}
		Ok(mip_removed)
	}
}
