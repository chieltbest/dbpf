use std::cmp::max;
use std::fmt::Debug;
use binrw::{args, BinResult, binrw, Error};
use crate::common::BigString;
use crate::internal_file::resource_collection::{FileName, ResourceBlockVersion};

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TextureFormat {
    RawBGRA = 1,
    RawBGR = 2,
    Alpha = 3,
    DXT1 = 4,
    #[default]
    DXT3 = 5,
    Grayscale = 6,
    AltBGRA = 7,
    DXT5 = 8,
    AltBGR = 9,
}

impl TextureFormat {
    /// get the size that a texture will be in memory when encoded in this texture format
    pub fn compressed_size(&self, width: usize, height: usize) -> usize {
        match self {
            TextureFormat::RawBGRA |
            TextureFormat::AltBGRA => width * height * 4,
            TextureFormat::RawBGR |
            TextureFormat::AltBGR => width * height * 3,
            TextureFormat::Grayscale |
            TextureFormat::Alpha => width * height,
            TextureFormat::DXT1 => texpresso::Format::Bc1.compressed_size(width, height),
            TextureFormat::DXT3 => texpresso::Format::Bc2.compressed_size(width, height),
            TextureFormat::DXT5 => texpresso::Format::Bc3.compressed_size(width, height),
        }
    }

    pub fn decompress(&self, data: &[u8], width: usize, height: usize, output: &mut [u8]) {
        match self {
            TextureFormat::RawBGRA |
            TextureFormat::AltBGRA => {
                output.copy_from_slice(
                    data.chunks_exact(4)
                        .map(|px| {
                            [px[2], px[1], px[0], px[3]]
                        })
                        .flatten()
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::RawBGR |
            TextureFormat::AltBGR => {
                output.copy_from_slice(
                    data.chunks_exact(3)
                        .map(|px| {
                            [px[2], px[1], px[0], 0xFF]
                        })
                        .flatten()
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::Grayscale => {
                output.copy_from_slice(
                    data.iter()
                        .map(|px| {
                            [*px, *px, *px, 0xFF]
                        })
                        .flatten()
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::Alpha => {
                output.copy_from_slice(
                    data.iter()
                        .map(|px| {
                            [0, 0, 0, *px]
                        })
                        .flatten()
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::DXT1 => texpresso::Format::Bc1.decompress(data, width, height, output),
            TextureFormat::DXT3 => texpresso::Format::Bc2.decompress(data, width, height, output),
            TextureFormat::DXT5 => texpresso::Format::Bc3.decompress(data, width, height, output),
        }
    }


    pub fn compress(&self, data: &[u8], width: usize, height: usize, output: &mut [u8]) {
        match self {
            TextureFormat::RawBGRA |
            TextureFormat::AltBGRA => {
                output.copy_from_slice(
                    data.chunks_exact(4)
                        .map(|px| {
                            [px[3], px[2], px[1], px[0]]
                        })
                        .flatten()
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::RawBGR |
            TextureFormat::AltBGR => {
                output.copy_from_slice(
                    data.chunks_exact(4)
                        .map(|px| {
                            [px[3], px[2], px[1]]
                        })
                        .flatten()
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::Grayscale => {
                output.copy_from_slice(
                    data.chunks_exact(4)
                        .map(|px| {
                            ((px[0] as u16) + (px[1] as u16) + (px[2] as u16)
                                / 3) as u8
                        })
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::Alpha => {
                output.copy_from_slice(
                    data.chunks_exact(4)
                        .map(|px| {
                            px[3]
                        })
                        .collect::<Vec<u8>>()
                        .as_slice());
            }
            TextureFormat::DXT1 => texpresso::Format::Bc1.compress(data, width, height, texpresso::Params {
                algorithm: texpresso::Algorithm::IterativeClusterFit,
                ..Default::default()
            }, output),
            TextureFormat::DXT3 => texpresso::Format::Bc2.compress(data, width, height, texpresso::Params {
                algorithm: texpresso::Algorithm::IterativeClusterFit,
                ..Default::default()
            }, output),
            TextureFormat::DXT5 => texpresso::Format::Bc3.compress(data, width, height, texpresso::Params {
                algorithm: texpresso::Algorithm::IterativeClusterFit,
                ..Default::default()
            }, output),
        }
    }
}

#[binrw]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct EmbeddedTextureResourceMipLevel {
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    num_input_bytes: u32,
    #[br(args { count: num_input_bytes as usize })]
    pub data: Vec<u8>,
}

impl EmbeddedTextureResourceMipLevel {
    pub fn decompress(&self, width: usize, height: usize, format: TextureFormat) -> BinResult<Vec<u8>> {
        let mut out = vec![0; width * height * 4];
        // move this logic to binread?
        let compressed_size = format.compressed_size(width, height);
        if compressed_size != self.data.len() {
            eprintln!("decompress failed: {:?} {} {compressed_size} {} {}", format, self.data.len(), width, height);
            return Err(Error::AssertFail {
                pos: 0,
                message: "Mipmap level has bad size".to_string(),
            });
        }
        format.decompress(&self.data, width, height, &mut out);
        Ok(out)
    }

    pub fn compress(&mut self, width: usize, height: usize, format: TextureFormat, data: &[u8]) {
        self.data.resize(format.compressed_size(width, height), 0);
        format.compress(data, width, height, &mut self.data);
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
    LIFOFile {
        file_name: BigString,
    },
}

#[binrw]
#[brw(import {version: ResourceBlockVersion, mip_levels: u32})]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextureResourceTexture {
    #[br(if (matches ! (version, ResourceBlockVersion::V9), mip_levels))]
    #[bw(if (matches ! (version, ResourceBlockVersion::V9)), calc = entries.len() as u32)]
    pub mip_levels: u32,

    #[br(count = mip_levels)]
    pub entries: Vec<TextureResourceData>,

    // 0xFF000000 or 0xFFFFFFFF when not uploaded online
    pub creator_id: u32,
    #[brw(if (matches ! (version, ResourceBlockVersion::V9)))]
    pub format_flag: u32,
}

#[binrw]
#[brw(import {version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default)]
pub struct TextureResource {
    pub file_name: FileName,

    pub width: u32,
    pub height: u32,

    pub format: TextureFormat,
    // TODO calc from texture vec len
    pub mip_levels: u32,
    pub purpose: f32,
    #[br(temp)]
    #[bw(calc = textures.len() as u32)]
    num_textures: u32,
    pub unknown: u32,

    #[br(if (matches ! (version, ResourceBlockVersion::V9)))]
    pub file_name_repeat: BigString,

    #[br(args {count: num_textures as usize, inner: args ! {version, mip_levels}})]
    #[bw(args {version, mip_levels: * mip_levels})]
    pub textures: Vec<TextureResourceTexture>,
}

pub struct DecodedTexture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl TextureResource {
    /// decompress a single texture given by its texture and mip index
    /// the mip index goes from 0 (largest) to mip_levels (smallest)
    /// if a mip level is represented as a LIFOStream then this function will return None
    pub fn decompress(&self, texture_index: usize, mip_index: usize) -> BinResult<DecodedTexture> {
        let TextureResourceData::Embedded(ref data)
            = self.textures[texture_index].entries[mip_index] else {
            return Err(Error::AssertFail {
                pos: 0,
                message: "Mipmap entry is a LIFO file".to_string(),
            });
        };
        let mip_index = (self.mip_levels - 1) as usize - mip_index;
        let width = max(self.width >> mip_index, 1) as usize;
        let height = max(self.height >> mip_index, 1) as usize;
        let data = data.decompress(width, height, self.format)?;
        Ok(DecodedTexture {
            data,
            width,
            height,
        })
    }

    pub fn decompress_all(&self) -> Vec<Vec<BinResult<DecodedTexture>>> {
        (0..self.textures.len()).into_iter().map(|texture_index| {
            (0..self.textures[texture_index].entries.len()).into_iter().map(|mip_index| {
                self.decompress(texture_index, mip_index)
            }).collect()
        }).collect()
    }

    pub fn compress(&mut self) {
        todo!()
    }

    pub fn compress_all(&mut self) {
        todo!()
    }

    /// remove n mipmap levels, starting from the largest first, effectively decreasing the size of the texture
    /// the number of mipmap levels will be reduced by mip_levels
    pub fn remove_largest_mip_levels(&mut self, mip_levels: usize) {
        self.width = max(self.width >> mip_levels, 1);
        self.height = max(self.height >> mip_levels, 1);
        self.mip_levels -= mip_levels as u32;
        self.textures.iter_mut().for_each(|texture| {
            texture.entries.truncate(self.mip_levels as usize);
        });
    }

    /// truncate the number of mipmap levels, starting from the smallest first
    /// mip_levels is the number of mipmap levels that should be left after the operation
    pub fn remove_smallest_mip_levels(&mut self, mip_levels: usize) {
        self.textures.iter_mut().for_each(|texture| {
            texture.entries.drain(..(self.mip_levels as usize - mip_levels));
        });
        self.mip_levels = mip_levels as u32;
    }
}
