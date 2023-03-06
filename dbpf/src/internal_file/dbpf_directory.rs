use binrw::{binrw, until_eof};
use crate::filetypes::DBPFFileType;
use crate::header_v1::{InstanceId, IndexMinorVersion};

#[binrw]
#[brw(import{version: IndexMinorVersion})]
#[derive(Clone, Debug)]
pub struct DBPFDirectoryEntry {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    #[brw(args {version})]
    pub instance_id: InstanceId,
    pub decompressed_size: u32,
}

#[binrw]
#[brw(import{version: IndexMinorVersion}, little)]
#[derive(Clone, Debug)]
pub struct DBPFDirectory {
    #[br(parse_with = until_eof, args {version})]
    #[bw(args {version})]
    pub entries: Vec<DBPFDirectoryEntry>,
}
