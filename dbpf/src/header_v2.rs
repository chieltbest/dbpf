use std::io::{Read, Seek};
use binrw::{args, binread, BinResult, BinRead, BinWrite};
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;
use crate::filetypes::DBPFFileType;
use crate::lazy_file_ptr::{LazyFilePtr, Zero};
use crate::{Timestamp, UserVersion, Version, HEADER_SIZE, Header, Index, IndexEntry, DBPFError, CompressionType};
use crate::internal_file::{FileData, FileDataBinReadArgs};

#[binread]
#[derive(Clone, Debug)]
pub struct HeaderV2 {
    #[brw(assert(matches ! (version, Version::V2(_))))]
    pub version: Version,
    pub user_version: UserVersion,
    pub flags: u32,
    pub created: Timestamp,
    pub modified: Timestamp,
    #[br(temp)]
    index_version: u32,
    #[br(temp)]
    index_entry_count: u32,
    #[br(temp)]
    index_location: u32,
    #[br(temp)]
    index_size: u32,
    #[br(temp)]
    hole_index_entry_count: u32,
    #[br(temp)]
    hole_index: u32,
    #[br(temp)]
    hole_index_size: u32,
    #[br(temp)]
    #[bw(calc = 3)]
    index_minor_version: u32,

    #[br(args { inner: args ! { count: index_entry_count as usize }})]
    #[brw(assert(index_entry_count == 0 || index.ptr >= HEADER_SIZE, "index count was {} (non-zero) while index location was {}", index_entry_count, index.ptr))]
    pub index: LazyFilePtr<u32, IndexV2, IndexV2BinReadArgs>,
}

#[bitfield]
#[derive(BinRead, BinWrite, Clone, Copy, Debug)]
#[br(map = Self::from_bytes)]
#[bw(map = | & x | Self::into_bytes(x))]
pub struct IndexType {
    fixed_type: bool,
    fixed_group: bool,
    fixed_instance: bool,
    #[skip]
    unused: B29,
}

#[binread]
#[br(import { count: usize })]
#[derive(Clone, Debug)]
pub struct IndexV2 {
    pub index_type: IndexType,

    #[brw(if (index_type.fixed_type()))]
    pub type_id: Option<DBPFFileType>,
    #[brw(if (index_type.fixed_group()))]
    pub group_id: Option<u32>,
    #[brw(if (index_type.fixed_instance()))]
    pub instance_id_ex: Option<u32>,

    #[br(args { count: count, inner: args ! {index_type: index_type.clone(), type_id, group_id, instance_id_ex} })]
    pub entries: Vec<IndexEntryV2>,
}

#[bitfield]
#[derive(BinRead, BinWrite, Clone, Copy, Debug)]
#[br(map = Self::from_bytes)]
#[bw(map = | & x | Self::into_bytes(x))]
pub struct FileSize {
    size: B31,
    ext_compressed: bool,
}

#[binread]
#[brw(import {
index_type: IndexType,
type_id: Option < DBPFFileType >,
group_id: Option < u32 >,
instance_id_ex: Option < u32 >})]
#[derive(Clone, Debug)]
pub struct IndexEntryV2 {
    #[br(calc = index_type)]
    #[bw(ignore)]
    index_type: IndexType,

    #[brw(if (!index_type.fixed_type(), type_id.unwrap()))]
    pub type_id: DBPFFileType,
    #[brw(if (!index_type.fixed_group(), group_id.unwrap()))]
    pub group_id: u32,
    #[brw(if (!index_type.fixed_instance(), instance_id_ex.unwrap()))]
    pub instance_id_ex: u32,
    pub instance_id: u32,

    pub file_location: u32,
    pub file_size: FileSize,
    pub decompressed_size: u32,

    #[brw(if(file_size.ext_compressed(), CompressionType::Uncompressed))]
    pub compression_type: CompressionType,
    #[brw(if(file_size.ext_compressed(), 1))]
    pub committed: u16,

    #[br(args {
    offset: u32::from(file_location) as u64,
    inner: args ! {
    count: file_size.size() as usize,
    compression_type,
    decompressed_size,
    type_id
    }})]
    pub data: LazyFilePtr<Zero, FileData, FileDataBinReadArgs>,
}

impl Header for HeaderV2 {
    type Index = IndexV2;

    fn version(&mut self) -> &mut Version {
        &mut self.version
    }

    fn user_version(&mut self) -> &mut UserVersion {
        &mut self.user_version
    }

    fn flags(&mut self) -> &mut u32 {
        &mut self.flags
    }

    fn created(&mut self) -> &mut Timestamp {
        &mut self.created
    }

    fn modified(&mut self) -> &mut Timestamp {
        &mut self.modified
    }

    fn index<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut IndexV2> {
        self.index.get(reader)
    }
}

impl Index for IndexV2 {
    type IndexEntry = IndexEntryV2;

    fn entries(&mut self) -> Vec<&mut Self::IndexEntry> {
        self.entries.iter_mut().collect()
    }
}

impl IndexEntry for IndexEntryV2 {
    fn data<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut FileData> {
        self.data.get(reader)
    }

    fn compression_type(&self) -> CompressionType {
        self.compression_type.clone()
    }

    fn get_type(&self) -> &DBPFFileType {
        &self.type_id
    }

    fn set_type(&mut self, file_type: DBPFFileType) -> Result<(), DBPFError> {
        if self.index_type.fixed_type() {
            Err(DBPFError::FixedType)
        } else {
            self.type_id = file_type;
            Ok(())
        }
    }

    fn get_group(&self) -> u32 {
        self.group_id
    }

    fn set_group(&mut self, group: u32) -> Result<(), DBPFError> {
        if self.index_type.fixed_group() {
            Err(DBPFError::FixedGroup)
        } else {
            self.group_id = group;
            Ok(())
        }
    }

    fn get_instance(&self) -> u64 {
        (self.instance_id as u64) | ((self.instance_id_ex as u64) << 32)
    }

    fn set_instance(&mut self, instance: u64) -> Result<(), DBPFError> {
        let instance_high: u32 = (instance >> 32) as u32;
        let instance_low = instance as u32;
        if self.index_type.fixed_instance() &&
            instance_high != self.instance_id_ex {
            Err(DBPFError::FixedInstance)
        } else {
            self.instance_id_ex = instance_high;
            self.instance_id = instance_low;
            Ok(())
        }
    }
}
