pub mod common;

pub mod dbpf_directory;
pub mod property_set;
pub mod sim_outfits;
pub mod resource_collection;
pub mod behaviour_function;
pub mod text_list;

use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use binrw::{binread, BinRead, BinWrite, BinResult, binrw, NamedArgs};
use miniz_oxide::inflate::DecompressError;
use refpack::data::compression::CompressionOptions;
use refpack::format::{Maxis, Reference, SimEA};
use refpack::RefPackError;
use thiserror::Error;
use crate::CompressionType;
use crate::filetypes::{DBPFFileType, KnownDBPFFileType};
use crate::internal_file::property_set::PropertySet;
use crate::internal_file::resource_collection::ResourceCollection;
use crate::internal_file::sim_outfits::SimOutfits;
use crate::internal_file::text_list::TextList;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error(transparent)]
    RefPack(#[from] RefPackError),
    #[error("{0}")]
    ZLib(DecompressError),
    #[error(transparent)]
    BinResult(#[from] binrw::Error),
}

impl From<DecompressError> for CompressionError {
    fn from(value: DecompressError) -> Self {
        CompressionError::ZLib(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FileDataInternal {
    Compressed(CompressedFileData),
    Uncompressed(RawFileData),
    Decoded(DecodedFile),
}

#[derive(Clone, NamedArgs)]
pub struct FileDataBinReadArgs {
    count: usize,
    pub compression_type: CompressionType,
    pub decompressed_size: u32,
    type_id: DBPFFileType,
}

#[binread]
#[br(import_raw(args: FileDataBinReadArgs))]
#[derive(Clone, Debug)]
pub struct FileData {
    #[br(temp,
    args {
    count: args.count,
    compression_type: args.compression_type,
    decompressed_size: args.decompressed_size
    })]
    compressed: CompressedFileData,
    #[br(calc = args.type_id)]
    pub(crate) type_id: DBPFFileType,
    #[br(calc = FileDataInternal::Compressed(compressed))]
    pub(crate) data: FileDataInternal,
}

impl FileData {
    pub fn compressed(&mut self, compression_type: CompressionType) -> Result<&mut CompressedFileData, CompressionError> {
        match &mut self.data {
            FileDataInternal::Compressed(data)
            if data.compression_type == compression_type => {}
            _ => {
                let data = self.decompressed()?;
                let compressed = CompressedFileData::compress(std::mem::take(data), compression_type)?;
                self.data = FileDataInternal::Compressed(compressed);
            }
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
                self.data = FileDataInternal::Uncompressed(data.clone().decompress()?);
            }
            FileDataInternal::Decoded(ref mut data) => {
                self.data = FileDataInternal::Uncompressed(data.clone().to_bytes()?);
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
#[derive(Clone, Default)]
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
                    refpack::easy_compress::<Maxis>(&data.data, CompressionOptions::Optimal)?
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
                    // TODO add a force format override
                    refpack::easy_decompress::<Maxis>(&self.data)
                        .and_then(|v| {
                            // eprintln!("sims2");
                            Ok(v)
                        })
                        .or_else(|_| refpack::easy_decompress::<SimEA>(&self.data)
                            .and_then(|v| {
                                // eprintln!("sims3");
                                Ok(v)
                            }))
                        .or_else(|_| refpack::easy_decompress::<Reference>(&self.data)
                            .and_then(|v| {
                                // eprintln!("ref");
                                Ok(v)
                            }))?
                }
                CompressionType::ZLib => {
                    miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(
                        &self.data, self.decompressed_size as usize)?
                }
                CompressionType::Deleted => self.data,
                _ => todo!(),
            },
        })
    }
}

impl Debug for CompressedFileData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressedFileData")
            .field("compression_type", &self.compression_type)
            .field("decompressed_size", &self.decompressed_size)
            .field("compressed_bytes", &self.data.len())
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct RawFileData {
    pub data: Vec<u8>,
}

impl RawFileData {
    pub fn decode(&self, file_type: DBPFFileType) -> Option<BinResult<DecodedFile>> {
        DecodedFile::decode_bytes(&self.data, file_type)
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

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum DecodedFile {
    PropertySet(PropertySet),
    SimOutfits(SimOutfits),
    ResourceCollection(ResourceCollection),
    TextList(TextList),
}

impl DecodedFile {
    pub fn decode_bytes(data: &[u8], file_type: DBPFFileType) -> Option<BinResult<Self>> {
        let mut cursor = Cursor::new(data);
        match file_type {
            DBPFFileType::Known(KnownDBPFFileType::PropertySet) => {
                Some(PropertySet::read(&mut cursor).map(|r| DecodedFile::PropertySet(r)))
            }
            DBPFFileType::Known(KnownDBPFFileType::SimOutfits) => {
                Some(SimOutfits::read(&mut cursor).map(|r| DecodedFile::SimOutfits(r)))
            }
            DBPFFileType::Known(KnownDBPFFileType::TextureResource |
                                KnownDBPFFileType::MaterialDefinition) => {
                Some(ResourceCollection::read(&mut cursor).map(|r| DecodedFile::ResourceCollection(r)))
            }
            DBPFFileType::Known(KnownDBPFFileType::TextList |
                                KnownDBPFFileType::CatalogDescription |
                                KnownDBPFFileType::CatalogString |
                                KnownDBPFFileType::PieMenuStrings) => {
                Some(TextList::read(&mut cursor).map(|r| DecodedFile::TextList(r)))
            }
            _ => None,
        }
    }

    pub fn to_bytes(self) -> BinResult<RawFileData> {
        let mut data = Cursor::new(Vec::new());
        match self {
            DecodedFile::PropertySet(x) => x.write(&mut data)?,
            DecodedFile::SimOutfits(x) => x.write(&mut data)?,
            DecodedFile::ResourceCollection(x) => x.write(&mut data)?,
            DecodedFile::TextList(x) => x.write(&mut data)?,
        }
        // TODO write error handling?
        Ok(RawFileData { data: data.into_inner() })
    }
}
