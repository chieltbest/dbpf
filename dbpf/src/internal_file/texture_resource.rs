use std::fmt::{Debug, Formatter};
use std::io::{Read, Seek, Write};
use binrw::{args, BinRead, BinReaderExt, BinResult, binrw, BinWrite, BinWriterExt, Endian, Error, NamedArgs};
use crate::common::BigString;
use crate::internal_file::resource_collection::{FileName, ResourceBlockVersion};

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum TextureFormat {
    RawARGB = 1,
    RawRGB = 2,
    Alpha = 3,
    DXT1 = 4,
    #[default]
    DXT3 = 5,
    Grayscale = 6,
    Alt32Bit = 7,
    DXT5 = 8,
    Alt24Bit = 9,
}

impl TextureFormat {
    pub fn get_transcode_format(&self) -> texpresso::Format {
        match self {
            TextureFormat::DXT1 => texpresso::Format::Bc1,
            TextureFormat::DXT3 => texpresso::Format::Bc2,
            TextureFormat::DXT5 => texpresso::Format::Bc3,
            _ => todo!()
        }
    }
}

#[derive(NamedArgs)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EmbeddedTextureResourceDataBinArgs {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
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
        let num_input_bytes: u32 = reader.read_type(endian)?;
        let mut data = vec![0u8; num_input_bytes as usize];
        reader.read_exact(data.as_mut_slice())?;
        let format = args.format.get_transcode_format();

        let mut new = Self {
            data: vec![],
            width: args.width,
            height: args.height,
        };
        while format.compressed_size(new.width as usize, new.height as usize)
            != num_input_bytes as usize {
            new.width /= 2;
            new.height /= 2;
            if new.width == 0 || new.height == 0 {
                return Err(Error::AssertFail {
                    pos: reader.stream_position()?,
                    message: "Couldn't find mipmap level".to_string(),
                });
            }
        }
        new.data = vec![0u8; (new.width * new.height * 4) as usize];
        format.decompress(data.as_slice(),
                          new.width as usize,
                          new.height as usize,
                          new.data.as_mut_slice());
        Ok(new)
    }
}

impl BinWrite for EmbeddedTextureResourceMipLevel {
    type Args<'a> = EmbeddedTextureResourceDataBinArgs;

    fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: Endian, args: Self::Args<'_>) -> BinResult<()> {
        let format = args.format.get_transcode_format();
        let len = format.compressed_size(self.width as usize, self.height as usize);
        let mut data = vec![0u8; len];
        format.compress(
            self.data.as_slice(),
            self.width as usize,
            self.height as usize,
            texpresso::Params {
                algorithm: texpresso::Algorithm::IterativeClusterFit,
                ..Default::default()
            },
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
#[brw(import {width: u32, height: u32, format: TextureFormat})]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureResourceData {
    #[brw(magic = 0u8)]
    Embedded(
        #[brw(args {width, height, format})]
        EmbeddedTextureResourceMipLevel),
    #[brw(magic = 1u8)]
    LIFOFile {
        file_name: BigString,
    },
}

#[binrw]
#[brw(import {version: ResourceBlockVersion, mip_levels: u32, width: u32, height: u32, format: TextureFormat})]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextureResourceTexture {
    #[br(if (matches ! (version, ResourceBlockVersion::V9), mip_levels))]
    #[bw(if (matches ! (version, ResourceBlockVersion::V9)), calc = entries.len() as u32)]
    pub mip_levels: u32,

    #[br(args {count: mip_levels as usize, inner: args ! {width, height, format}})]
    #[bw(args {width, height, format})]
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
