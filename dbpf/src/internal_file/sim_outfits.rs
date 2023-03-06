use binrw::{args, binrw};
use crate::filetypes::DBPFFileType;
use crate::header_v1::{InstanceId, IndexMinorVersion};

#[binrw]
#[brw(import {version: IndexMinorVersion})]
#[derive(Clone, Debug)]
pub struct Entry {
    pub type_id: DBPFFileType,
    pub group_id: u32,
    #[brw(args{version})]
    pub instance_id: InstanceId,
}

#[binrw]
#[brw(magic = 0xDEADBEEFu32, little)]
#[derive(Clone, Debug)]
pub struct SimOutfits {
    pub version: IndexMinorVersion,
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    count: u32,

    #[br(count = count, args {inner: args! {version}})]
    #[bw(args {version: *version})]
    pub entries: Vec<Entry>,
}
