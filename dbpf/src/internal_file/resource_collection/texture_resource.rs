use std::cmp::max;
use std::fmt::{Debug, Formatter};
use std::io::{Read, Seek, Write};
use binrw::{args, BinRead, BinReaderExt, BinResult, binrw, BinWrite, BinWriterExt, Endian, Error, NamedArgs, writer};
use binrw::helpers::args_iter;
use crate::common::BigString;
use crate::internal_file::resource_collection::{FileName, ResourceBlockVersion};

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
                            [px[3], px[2], px[1], px[0]]
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
                            [0xFF, px[2], px[1], px[0]]
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

#[derive(NamedArgs)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EmbeddedTextureResourceDataBinArgs {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_level: u32,
}

#[derive(Clone, Eq, PartialEq)]
pub struct EmbeddedTextureResourceMipLevel {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl BinRead for EmbeddedTextureResourceMipLevel {
    type Args<'a> = EmbeddedTextureResourceDataBinArgs;

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, args: Self::Args<'_>) -> BinResult<Self> {
        // TODO get mip level as arg

        let num_input_bytes: u32 = reader.read_type(endian)?;
        let mut data = vec![0u8; num_input_bytes as usize];
        reader.read_exact(data.as_mut_slice())?;

        let mut new = Self {
            data: vec![],
            width: max(args.width >> args.mip_level, 1),
            height: max(args.height >> args.mip_level, 1),
        };
        let compressed_size = args.format.compressed_size(new.width as usize, new.height as usize);
        if compressed_size != num_input_bytes as usize {
            eprintln!("{:?} {num_input_bytes} {compressed_size} {} {} {} {} {}", args.format, args.width, args.height, args.mip_level, new.width, new.height);
            return Err(Error::AssertFail {
                pos: reader.stream_position()?,
                message: "Mipmap level has bad size".to_string(),
            });
        }
        new.data = vec![0u8; (new.width * new.height * 4) as usize];
        args.format.decompress(data.as_slice(),
                               new.width as usize,
                               new.height as usize,
                               new.data.as_mut_slice());
        Ok(new)
    }
}

impl BinWrite for EmbeddedTextureResourceMipLevel {
    type Args<'a> = EmbeddedTextureResourceDataBinArgs;

    fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: Endian, args: Self::Args<'_>) -> BinResult<()> {
        // TODO check mip level matches size
        let len = args.format.compressed_size(self.width as usize, self.height as usize);
        let mut data = vec![0u8; len];
        args.format.compress(
            self.data.as_slice(),
            self.width as usize,
            self.height as usize,
            data.as_mut_slice());
        writer.write_type(&(len as u32), endian)?;
        writer.write_type(&data, endian)
    }
}

impl Debug for EmbeddedTextureResourceMipLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out_strings = vec![String::new(); 4];
        for channel in 0..4 {
            for y in 0..self.height {
                for x in 0..self.width {
                    out_strings[channel].push_str(
                        match self.data[((y * self.width + x) * 4 + channel as u32) as usize] {
                            0..=15 => " ",
                            16..=31 => ".",
                            32..=47 => "-",
                            48..=63 => "+",
                            64..=79 => "x",
                            80..=95 => "X",
                            96..=111 => "#",
                            _ => "@",
                        })
                }
                out_strings[channel].push_str("\n");
            }
        }
        struct DisplayString<'a> {
            str: &'a String,
        }
        impl<'a> Debug for DisplayString<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                writeln!(f, "")?;
                write!(f, "{}", self.str)
            }
        }
        f.debug_struct("EmbeddedTextureResourceMipLevel")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("red", &DisplayString { str: &out_strings[0] })
            .field("green", &DisplayString { str: &out_strings[1] })
            .field("blue", &DisplayString { str: &out_strings[2] })
            .field("alpha", &DisplayString { str: &out_strings[3] })
            .finish()
    }
}

#[binrw]
#[brw(import {width: u32, height: u32, format: TextureFormat, mip_level: u32})]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureResourceData {
    #[brw(magic = 0u8)]
    Embedded(
        #[brw(args {width, height, format, mip_level})]
        EmbeddedTextureResourceMipLevel),
    #[brw(magic = 1u8)]
    LIFOFile {
        file_name: BigString,
    },
}

#[writer(writer, endian)]
fn write_mips_vec(vec: &Vec<TextureResourceData>, width: u32, height: u32, format: TextureFormat, mip_levels: u32) -> BinResult<()> {
    for (i, elem) in vec.iter().enumerate() {
        elem.write_options(writer, endian, args! {
            width,
            height,
            format,
            mip_level: mip_levels - (i as u32) - 1,
        })?;
    }
    Ok(())
}

#[binrw]
#[brw(import {version: ResourceBlockVersion, mip_levels: u32, width: u32, height: u32, format: TextureFormat})]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextureResourceTexture {
    #[br(if (matches ! (version, ResourceBlockVersion::V9), mip_levels))]
    #[bw(if (matches ! (version, ResourceBlockVersion::V9)), calc = entries.len() as u32)]
    pub mip_levels: u32,

    // #[br(args {count: mip_levels as usize, inner: args ! {width, height, format}})]
    #[br(parse_with = args_iter((0..mip_levels).into_iter().map(| i | -> TextureResourceDataBinReadArgs {
    args ! {width, height, format, mip_level: mip_levels - i - 1}
    })))]
    #[bw(write_with = write_mips_vec, args(width, height, format, mip_levels))]
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
    pub mip_levels: u32,
    pub purpose: f32,
    #[br(temp)]
    #[bw(calc = textures.len() as u32)]
    num_textures: u32,
    pub unknown: u32,

    #[br(if (matches ! (version, ResourceBlockVersion::V9)))]
    pub file_name_repeat: BigString,

    #[br(args {count: num_textures as usize, inner: args ! {version, mip_levels, width, height, format}})]
    #[bw(args {version, mip_levels: * mip_levels, width: * width, height: * height, format: * format})]
    pub textures: Vec<TextureResourceTexture>,
}
