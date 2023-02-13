pub mod filetypes;
// pub mod lazy_file_ptr;
mod lazy_file_ptr;

use std::fmt::{Debug, Formatter};
use std::num::NonZeroU32;
use binrw::{binread, binrw, io::SeekFrom, VecArgs};
use crate::filetypes::DBPFFileType;
use crate::lazy_file_ptr::LazyFilePtr;

const HEADER_SIZE: u32 = 0x60;

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
}

#[binrw]
#[derive(Debug)]
pub struct UserVersion {
    major: u32,
    minor: u32,
}

#[binrw]
#[derive(Debug)]
pub struct Timestamp(u32);

#[binrw]
#[brw(repr = u32)]
#[derive(Debug)]
pub enum IndexVersion {
    Default = 7,
    Spore = 0,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug)]
pub enum IndexMinorVersion {
    V1 = 0,
    V2 = 1,
    V3 = 2,
    V4 = 3,
}

#[binread]
#[derive(Debug)]
pub struct HeaderV1 {
    pub version: Version,
    pub user_version: UserVersion,
    pub flags: u32,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub index_version: IndexVersion,
    #[br(temp)]
    index_entry_count: u32,
    #[brw(args { inner: VecArgs {count: index_entry_count as usize, inner: ()} })]
    #[brw(assert(index_entry_count == 0 || index.ptr >= HEADER_SIZE, "index count was {} (non-zero) while index location was {}", index_entry_count, index.ptr))]
    pub index: LazyFilePtr<u32, Vec<IndexEntryV1>, VecArgs<()>>,
    #[br(temp)] // , assert(index_size == index_entry_count * 24)
    index_size: u32,
    #[br(temp)]
    hole_index_entry_count: u32,
    #[brw(args { inner: VecArgs {count: hole_index_entry_count as usize, inner: ()} })]
    #[brw(assert(hole_index_entry_count == 0 || hole_index.ptr >= HEADER_SIZE))]
    pub hole_index: LazyFilePtr<u32, Vec<HoleIndexEntry>, VecArgs<()>>,
    #[br(temp)]
    #[brw(assert(hole_index_size == hole_index_entry_count * 8))]
    hole_index_size: u32,
    pub index_minor_version: IndexMinorVersion,
    #[br(if (matches ! (version, Version::V2(_))))]
    pub index_offset_v2: u32,
}

#[binread]
#[derive(Debug)]
pub struct IndexEntryV1 {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    pub instance_id: u64,
    #[br(temp)]
    location: u32,
    #[br(temp)]
    size: u32,
    #[brw(seek_before = SeekFrom::Current(- 8), restore_position)]
    #[br(args { inner: FileDataBinReadArgs {count: size as usize} })]
    pub data: LazyFilePtr<NonZeroU32, FileData, FileDataBinReadArgs>,
}

#[binrw]
#[br(import { count: usize })]
pub struct FileData {
    #[br(count = count)]
    pub data: Vec<u8>,
}

impl Debug for FileData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FileData {{")?;
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
#[derive(Debug)]
pub struct HoleIndexEntry {
    pub location: u32,
    pub size: u32,
}

#[binread]
#[brw(magic = b"DBPF", little)]
#[derive(Debug)]
pub struct DBPFFile {
    #[brw(pad_size_to = HEADER_SIZE)]
    pub header: HeaderV1,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
