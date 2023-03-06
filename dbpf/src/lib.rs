extern crate core;

pub mod filetypes;
mod lazy_file_ptr;
pub mod header_v1;
pub mod header_v2;
pub mod internal_file;
pub mod common;

use std::io::{Read, Seek};
use std::num::TryFromIntError;
use binrw::{BinResult, binrw};
use crate::filetypes::DBPFFileType;
use crate::internal_file::FileData;

use binrw::{binread};
use crate::header_v1::HeaderV1;
use crate::header_v2::HeaderV2;

#[binread]
#[brw(magic = b"DBPF", little)]
#[bw(pad_size_to = header_common::HEADER_SIZE)]
#[derive(Clone, Debug)]
pub enum DBPFFile {
    HeaderV1(HeaderV1),
    HeaderV2(HeaderV2),
}

pub const HEADER_SIZE: u32 = 0x60;

#[binrw]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Version {
    #[brw(magic = 1u32)]
    V1(V1Minor),
    #[brw(magic = 2u32)]
    V2(V2Minor),
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum V1Minor {
    M0 = 0,
    M1 = 1,
    M2 = 2,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum V2Minor {
    M0 = 0,
    M1 = 1,
}

#[binrw]
#[derive(Copy, Clone, Debug)]
pub struct UserVersion {
    major: u32,
    minor: u32,
}

#[binrw]
#[derive(Copy, Clone, Debug)]
pub struct Timestamp(u32);

#[binrw]
#[brw(repr = u16)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum CompressionType {
    #[default]
    Uncompressed = 0x0000,
    Streamable = 0xFFFE,
    RefPack = 0xFFFF,
    Deleted = 0xFFE0,
    ZLib = 0x5A42,
}

#[derive(Debug)]
pub enum DBPFError {
    FixedType,
    FixedGroup,
    FixedInstance,

    BadInt(TryFromIntError),
    BadFormat(binrw::Error),
}

pub trait Header {
    type Index: Index;

    fn version(&mut self) -> &mut Version;
    fn user_version(&mut self) -> &mut UserVersion;
    fn flags(&mut self) -> &mut u32;
    fn created(&mut self) -> &mut Timestamp;
    fn modified(&mut self) -> &mut Timestamp;

    fn index<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut Self::Index>;
}

pub trait Index {
    type IndexEntry: IndexEntry;

    fn entries(&mut self) -> Vec<&mut Self::IndexEntry>;
}

pub trait IndexEntry {
    fn data<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut FileData>;

    fn compression_type(&self) -> CompressionType;

    fn get_type(&self) -> &DBPFFileType;
    fn set_type(&mut self, file_type: DBPFFileType) -> Result<(), DBPFError>;

    fn get_group(&self) -> u32;
    fn set_group(&mut self, group: u32) -> Result<(), DBPFError>;

    fn get_instance(&self) -> u64;
    fn set_instance(&mut self, instance: u64) -> Result<(), DBPFError>;
}
