pub mod cpf;
pub mod xml;

pub mod dbpf_directory;
pub mod property_set;
pub mod sim_outfits;
mod resource_collection;
mod texture_resource;

use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use binrw::{binread, BinRead, BinWrite, BinResult, binrw, NamedArgs};
use miniz_oxide::inflate::DecompressError;
use refpack::format::{Reference, Simcity4, TheSims12, TheSims34};
use refpack::RefPackError;
use thiserror::Error;
use crate::CompressionType;
use crate::filetypes::{DBPFFileType, KnownDBPFFileType};
use crate::internal_file::property_set::PropertySet;
use crate::internal_file::resource_collection::ResourceCollection;
use crate::internal_file::sim_outfits::SimOutfits;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error(transparent)]
    RefPack(#[from] RefPackError),
    #[error("{0}")]
    ZLib(DecompressError),
    #[error(transparent)]
    BinResult(#[from] binrw::Error)
}

impl From<DecompressError> for CompressionError {
    fn from(value: DecompressError) -> Self {
        CompressionError::ZLib(value)
    }
}

#[derive(Clone, Debug)]
enum FileDataInternal {
    Compressed(CompressedFileData),
    Uncompressed(RawFileData),
    Decoded(DecodedFile),
}

#[derive(Clone, NamedArgs)]
pub struct FileDataBinReadArgs {
    count: usize,
    pub compression_type: CompressionType,
    decompressed_size: u32,
    type_id: DBPFFileType,
}

#[binread]
#[br(import_raw(args: FileDataBinReadArgs))]
#[derive(Clone, Debug)]
pub struct FileData {
    #[br(temp, postprocess_now,
    args {
    count: args.count,
    compression_type: args.compression_type,
    decompressed_size: args.decompressed_size
    })]
    compressed: CompressedFileData,
    #[br(calc = args.type_id)]
    #[bw(ignore)]
    type_id: DBPFFileType,
    #[br(calc = FileDataInternal::Compressed(compressed))]
    data: FileDataInternal,
}

impl FileData {
    pub fn compressed(&mut self, compression_type: CompressionType) -> Result<&mut CompressedFileData, CompressionError> {
        match &mut self.data {
            FileDataInternal::Compressed(data)
            if data.compression_type != compression_type => {
                let data = self.decompressed()?;
                let compressed = CompressedFileData::compress(std::mem::take(data), compression_type)?;
                self.data = FileDataInternal::Compressed(compressed);
            }
            _ => {}
        }
        match &mut self.data {
            FileDataInternal::Compressed(data) => Ok(data),
            _ => unreachable!(),
        }
    }

    /// Decompresses the data if it is not already, and then returns a reference to that data
    /// The decompressed data will be stored for future calls
    pub fn decompressed(&mut self) -> Result<&mut RawFileData, CompressionError> {
        match self.data {
            FileDataInternal::Compressed(ref mut data) => {
                self.data = FileDataInternal::Uncompressed(std::mem::take(data).decompress()?);
            }
            FileDataInternal::Decoded(ref mut data) => {
                self.data = FileDataInternal::Uncompressed(std::mem::take(data).to_bytes());
            }
            _ => {}
        }
        match self.data {
            FileDataInternal::Uncompressed(ref mut data) => Ok(data),
            _ => unreachable!(),
        }
    }

    pub fn decoded(&mut self) -> Result<Option<&mut DecodedFile>, CompressionError> {
        match self.data {
            FileDataInternal::Decoded(_) => {}
            _ => {
                let type_id = self.type_id;
                let decompressed = self.decompressed()?;
                match decompressed.decode(type_id) {
                    Some(data) => {
                        self.data = FileDataInternal::Decoded(data?);
                    }
                    None => return Ok(None),
                }
            }
        }
        match &mut self.data {
            FileDataInternal::Decoded(decoded) => Ok(Some(decoded)),
            _ => unreachable!(),
        }
    }
}

#[binrw]
#[br(import { count: usize, compression_type: CompressionType, decompressed_size: u32 })]
#[derive(Clone, Debug, Default)]
pub struct CompressedFileData {
    #[br(calc = compression_type)]
    #[bw(ignore)]
    pub compression_type: CompressionType,
    #[br(calc = decompressed_size)]
    #[bw(ignore)]
    pub decompressed_size: u32,
    #[br(count = count)]
    pub data: Vec<u8>,
}

impl CompressedFileData {
    fn compress(data: RawFileData, compression_type: CompressionType) -> Result<CompressedFileData, CompressionError> {
        Ok(CompressedFileData {
            compression_type,
            decompressed_size: data.data.len() as u32,
            data: match compression_type {
                CompressionType::Uncompressed => data.data,
                CompressionType::RefPack => {
                    refpack::easy_compress::<TheSims12>(&data.data)?
                    // TODO add a config switch for compression type
                }
                CompressionType::ZLib => {
                    miniz_oxide::deflate::compress_to_vec_zlib(&data.data, 10)
                }
                _ => todo!(),
            },
        })
    }

    fn decompress(self) -> Result<RawFileData, CompressionError> {
        Ok(RawFileData {
            data: match self.compression_type {
                CompressionType::Uncompressed => self.data,
                CompressionType::RefPack => {
                    // try all formats in the order of how restrictive they are
                    refpack::easy_decompress::<TheSims12>(&self.data)
                        .or_else(|_| refpack::easy_decompress::<Simcity4>(&self.data))
                        .or_else(|_| refpack::easy_decompress::<TheSims34>(&self.data))
                        .or_else(|_| refpack::easy_decompress::<Reference>(&self.data))?
                }
                CompressionType::ZLib => {
                    miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(
                        &self.data, self.decompressed_size as usize)?
                }
                _ => todo!(),
            },
        })
    }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct RawFileData {
    pub data: Vec<u8>,
}

impl RawFileData {
    pub fn decode(&self, file_type: DBPFFileType) -> Option<BinResult<DecodedFile>> {
        match file_type {
            DBPFFileType::Known(known) => {
                let mut cursor = Cursor::new(&self.data);
                match DecodedFile::read_args(&mut cursor, DecodedFileBinReadArgs { type_id: known }) {
                    // only if there are no variant matches on the top level, otherwise it's a backtrace
                    Ok(DecodedFile::Unknown) => { None }
                    decoded => Some(decoded),
                }
            }
            _ => None
        }
    }
}

impl Debug for RawFileData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RawFileData {{")?;
        let align = f.fill().to_string().repeat(4);
        if let Ok(str) = String::from_utf8(self.data.clone()) {
            writeln!(f, "{}{}", align, str)?;
        } else {
            let lines = self.data.chunks(16);
            for line in lines {
                let mut line_hex_str = String::new();
                for group in line.chunks(2) {
                    for byte in group {
                        line_hex_str.push_str(format!("{byte:02x}").as_str());
                    }
                    line_hex_str.push_str(" ");
                }
                writeln!(f,
                         "{}{line_hex_str:40}{}",
                         align,
                         line.iter().map(|&c| match c {
                             0..=31 => char::from_u32(0x2400 + c as u32).unwrap(),
                             127 => '␡',
                             128.. => char::REPLACEMENT_CHARACTER,
                             _ => c.into(),
                         }).collect::<String>())?;
            }
        }
        write!(f, "}}")
    }
}

#[binrw]
#[br(import {type_id: KnownDBPFFileType})]
#[brw(little)]
#[derive(Clone, Debug, Default)]
pub enum DecodedFile {
    #[br(pre_assert(matches ! (type_id, KnownDBPFFileType::PropertySet)))]
    PropertySet(PropertySet),
    #[br(pre_assert(matches ! (type_id, KnownDBPFFileType::SimOutfits)))]
    SimOutfits(SimOutfits),
    #[br(pre_assert(matches ! (type_id, KnownDBPFFileType::TextureResource)))]
    ResourceCollection(ResourceCollection),

    /// used only for internal moves
    #[default]
    // match all the other types, because otherwise error passing would break
    #[br(pre_assert(! matches ! (type_id,
    KnownDBPFFileType::PropertySet |
    KnownDBPFFileType::SimOutfits |
    KnownDBPFFileType::TextureResource)))]
    Unknown,
}

impl DecodedFile {
    pub fn to_bytes(self) -> RawFileData {
        let mut data = Cursor::new(Vec::new());
        self.write(&mut data).unwrap();
        // TODO write error handling?
        RawFileData { data: data.into_inner() }
    }
}
