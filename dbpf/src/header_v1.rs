use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};
use std::num::NonZeroU32;
use binrw::{args, binread, BinRead, BinResult, binrw, VecArgs};
use crate::{DBPFError, Header, Index, IndexEntry, Timestamp, UserVersion, Version, HEADER_SIZE, CompressionType};
use crate::filetypes::{DBPFFileType, KnownDBPFFileType};
use crate::lazy_file_ptr::{LazyFilePtr, Zero};
use crate::internal_file::{FileData, FileDataBinReadArgs};
use crate::internal_file::dbpf_directory::DBPFDirectory;


#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug)]
pub enum IndexVersion {
    Default = 7,
    Spore = 0,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug)]
pub enum IndexMinorVersion {
    V0 = 0,
    V1 = 1,
    V2 = 2,
}

#[binread]
#[derive(Clone, Debug)]
pub struct HeaderV1 {
    #[brw(assert(matches ! (version, Version::V1(_))))]
    pub version: Version,
    pub user_version: UserVersion,
    pub flags: u32,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub index_version: IndexVersion,
    #[br(temp)]
    index_entry_count: u32,
    #[br(temp)]
    index_location: u32,
    #[br(temp)]
    index_size: u32,
    #[br(temp)]
    hole_index_entry_count: u32,
    #[brw(args { inner: args ! {count: hole_index_entry_count as usize, inner: ()} })]
    #[brw(assert(hole_index_entry_count == 0 || hole_index.ptr >= HEADER_SIZE))]
    pub hole_index: LazyFilePtr<u32, Vec<HoleIndexEntry>, VecArgs<()>>,
    #[br(temp)]
    #[brw(assert(hole_index_size == hole_index_entry_count * 8))]
    hole_index_size: u32,
    pub index_minor_version: IndexMinorVersion,

    #[brw(args {
    offset: index_location as u64,
    inner: args ! {
    count: index_entry_count as usize,
    version: index_minor_version
    }})]
    #[brw(assert(index_size == index_entry_count * if matches ! (index_minor_version, IndexMinorVersion::V2) { 24 } else { 20 }))]
    #[brw(assert(index_entry_count == 0 || index_location >= HEADER_SIZE, "index count was {} (non-zero) while index location was {}", index_entry_count, index.ptr))]
    pub index: LazyFilePtr<Zero, IndexV1, IndexV1BinReadArgs>,
}

#[binread]
#[brw(import { count: usize, version: IndexMinorVersion })]
#[derive(Clone, Debug)]
pub struct IndexV1 {
    #[brw(args { count: count, inner: args ! { version: version } })]
    entries: Vec<IndexEntryV1>,
}

#[binrw]
#[brw(import { version: IndexMinorVersion })]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Debug)]
pub struct InstanceId {
    #[br(temp)]
    #[bw(calc(* id as u32))]
    id_lower: u32,
    #[br(temp)]
    #[bw(calc((id >> 32) as u32))]
    #[brw(if (matches ! (version, IndexMinorVersion::V2)))]
    id_upper: u32,
    #[br(calc(id_lower as u64 | ((id_upper as u64) << 32)))]
    #[bw(ignore)]
    pub id: u64,
}

#[binread]
#[brw(import { version: IndexMinorVersion })]
#[derive(Clone, Debug)]
pub struct IndexEntryV1 {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    #[brw(args { version: version })]
    pub instance_id: InstanceId,
    #[br(temp)]
    location: NonZeroU32,
    #[br(temp)]
    size: u32,

    #[brw(ignore)]
    pub compression: Option<CompressionType>,
    #[br(calc = size)]
    #[bw(ignore)]
    pub decompressed_size: u32,
    #[br(args {
    offset: u32::from(location) as u64,
    inner: args ! {
    count: size as usize,
    compression_type: CompressionType::Uncompressed,
    decompressed_size,
    type_id
    }})]
    pub data: LazyFilePtr<Zero, FileData, FileDataBinReadArgs>,
}

#[binrw]
#[derive(Copy, Clone, Debug)]
pub struct HoleIndexEntry {
    pub location: u32,
    pub size: u32,
}

impl Header for HeaderV1 {
    type Index = IndexV1;

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

    fn index<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut IndexV1> {
        let previous_read = self.index.is_read();
        let mut data = self.index.get(reader);
        if !previous_read {
            if let Ok(index) = &mut data {
                let mut error: BinResult<()> = Ok(());
                let mut compressed_entries = HashMap::new();

                index.entries.retain_mut(|entry| {
                    match entry.type_id {
                        DBPFFileType::Known(KnownDBPFFileType::DBPFDirectory)
                        if error.is_ok() => {
                            entry.compression = Some(CompressionType::Uncompressed);
                            let data_res = entry.data(reader).and_then(|data| {
                                let data = &mut data.decompressed().data;
                                let res: DBPFDirectory = DBPFDirectory::read_args(
                                    &mut Cursor::new(data), args! {
                                        version: self.index_minor_version
                                    })?;
                                for entry in res.entries {
                                    compressed_entries.insert(
                                        (entry.type_id, entry.group_id, entry.instance_id),
                                        entry.decompressed_size);
                                }
                                Ok(())
                            });
                            if let Err(err) = data_res {
                                error = Err(err);
                            }
                            false
                        }
                        _ => true,
                    }
                });
                error?;

                for entry in &mut index.entries {
                    if let Some(decompressed_size) =
                        compressed_entries.get(&(entry.type_id, entry.group_id, entry.instance_id)) {
                        entry.compression = Some(CompressionType::RefPack);
                        entry.decompressed_size = *decompressed_size;
                    } else {
                        entry.compression = Some(CompressionType::Uncompressed);
                    }
                }
            }
        }
        data
    }
}

impl Index for IndexV1 {
    type IndexEntry = IndexEntryV1;

    fn entries(&mut self) -> Vec<&mut IndexEntryV1> {
        self.entries.iter_mut().collect()
    }
}

impl IndexEntry for IndexEntryV1 {
    fn data<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut FileData> {
        self.data.args.inner.compression_type = self.compression.unwrap();
        self.data.get(reader)
    }

    fn compression_type(&self) -> CompressionType {
        if let Some(compression_type) = self.compression {
            compression_type
        } else {
            panic!()
        }
    }

    fn get_type(&self) -> &DBPFFileType {
        &self.type_id
    }

    fn set_type(&mut self, file_type: DBPFFileType) -> Result<(), DBPFError> {
        self.type_id = file_type;
        Ok(())
    }

    fn get_group(&self) -> u32 {
        self.group_id
    }

    fn set_group(&mut self, group: u32) -> Result<(), DBPFError> {
        self.group_id = group;
        Ok(())
    }

    fn get_instance(&self) -> u64 {
        self.instance_id.id
    }

    fn set_instance(&mut self, instance: u64) -> Result<(), DBPFError> {
        self.instance_id.id = instance;
        Ok(())
    }
}
