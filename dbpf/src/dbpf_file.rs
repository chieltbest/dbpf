use std::io::{Read, Seek};
use std::mem;
use binrw::{binread, BinRead, BinResult, binrw};
use crate::{Timestamp, UserVersion, Version, CompressionType, IndexVersion, IndexMinorVersion};
use crate::filetypes::DBPFFileType;
use crate::header_v1::HeaderV1;
use crate::header_v2::HeaderV2;
use crate::lazy_file_ptr::{LazyFilePtr, Zero};
use crate::internal_file::{FileData, FileDataBinReadArgs};


#[binread]
#[brw(magic = b"DBPF", little)]
#[bw(pad_size_to = HEADER_SIZE)]
#[derive(Clone, Debug)]
enum DBPFFileVersions {
    HeaderV1(HeaderV1),
    HeaderV2(HeaderV2),
}

#[derive(Clone, Debug, Default)]
pub struct DBPFFile {
    pub version: Version,
    pub user_version: UserVersion,
    pub flags: u32,
    pub created: Timestamp,
    pub modified: Timestamp,
    pub index_version: IndexVersion,
    pub index_minor_version: IndexMinorVersion,

    pub hole_index: Vec<HoleIndexEntry>,

    pub index: Vec<IndexEntry>,
}

#[derive(Clone, Debug)]
pub struct IndexEntry {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    pub instance_id: u64,

    pub compression: CompressionType,
    data: LazyFilePtr<Zero, FileData, FileDataBinReadArgs>,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default)]
pub struct HoleIndexEntry {
    pub location: u32,
    pub size: u32,
}

impl DBPFFile {
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        let read_file = DBPFFileVersions::read(reader)?;
        match read_file {
            DBPFFileVersions::HeaderV1(mut header) => {
                Ok(Self {
                    version: header.version,
                    user_version: header.user_version,
                    flags: header.flags,
                    created: header.created,
                    modified: header.modified,
                    index_version: header.index_version,
                    index_minor_version: header.index_minor_version,
                    hole_index: mem::take(header.hole_index.get(reader)?),

                    index: mem::take(&mut header.index(reader)?.entries).into_iter()
                        .map(|entry| IndexEntry {
                                type_id: entry.type_id,
                                group_id: entry.group_id,
                                instance_id: entry.instance_id.id,

                                compression: entry.compression.unwrap_or(CompressionType::Uncompressed),
                                data: entry.data,
                            }).collect(),
                })
            }
            DBPFFileVersions::HeaderV2(mut header) => {
                Ok(Self {
                    version: header.version,
                    user_version: header.user_version,
                    flags: header.flags,
                    created: header.created,
                    modified: header.modified,
                    hole_index: vec![],

                    index: mem::take(&mut header.index(reader)?.entries).into_iter()
                        .map(|entry| IndexEntry {
                            type_id: entry.type_id,
                            group_id: entry.group_id,
                            instance_id: (entry.instance_id_ex as u64) << 32 | entry.instance_id as u64,

                            compression: entry.compression_type,
                            data: entry.data,
                        }).collect(),

                    ..Self::default()
                })
            }
        }
    }
}

impl IndexEntry {
    pub fn data<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut FileData> {
        self.data.get(reader)
    }
}
