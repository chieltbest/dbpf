use binrw::{args, binrw};
use crate::common::BigString;
use crate::filetypes::{DBPFFileType, KnownDBPFFileType};
use crate::internal_file::resource_collection::material_definition::MaterialDefinition;
use crate::internal_file::resource_collection::texture_resource::TextureResource;

pub mod texture_resource;
pub mod material_definition;

#[binrw]
#[brw(import {type_id: DBPFFileType, version: ResourceBlockVersion})]
#[derive(Clone, Debug)]
pub enum ResourceData {
    #[br(pre_assert(matches ! (type_id, DBPFFileType::Known(KnownDBPFFileType::TextureResource))))]
    Texture(#[brw(args {version: version.clone()})] TextureResource),
    #[br(pre_assert(matches ! (type_id, DBPFFileType::Known(KnownDBPFFileType::MaterialDefinition))))]
    Material(#[brw(args {version: version.clone()})] MaterialDefinition),
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileName {
    #[br(assert("cSGResource".as_bytes() == c_sg_resource.data.as_slice()))]
    #[bw(calc(BigString::from("cSGResource".to_string())))]
    pub c_sg_resource: BigString,

    #[br(assert(block_id == 0))]
    pub block_id: u32,
    #[br(assert(version == 2))]
    pub version: u32,

    pub name: BigString,
}

#[binrw]
#[brw(magic = 0xFFFF0001u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResourceVersion;

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResourceBlockVersion {
    V7 = 7,
    V8 = 8,
    #[default]
    V9 = 9,
    V10 = 10,
    V11 = 11,
}

#[binrw]
#[brw(import {version: bool})]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileLink {
    pub group_id: u32,
    pub instance_id: u32,
    #[brw(if (version))]
    pub resource_id: u32,
    pub type_id: DBPFFileType,
}

#[binrw]
#[derive(Clone, Debug)]
pub struct ResourceEntry {
    pub name: BigString,

    pub type_id: DBPFFileType,
    pub version: ResourceBlockVersion,

    #[brw(args {type_id: type_id.clone(), version: version.clone()})]
    pub data: ResourceData,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default)]
pub struct ResourceCollection {
    #[br(try, temp)]
    #[bw(calc(version.then_some(ResourceVersion)))]
    version_res: Option<ResourceVersion>,
    #[br(calc(version_res.is_some()))]
    #[bw(ignore)]
    pub version: bool,

    #[br(temp)]
    #[bw(calc = links.len() as u32)]
    link_count: u32,
    #[br(args {count: link_count as usize, inner: args ! {version: version}})]
    #[bw(args {version: *version})]
    pub links: Vec<FileLink>,

    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    item_count: u32,
    #[br(count = item_count)]
    #[bw(calc = entries.iter().map(|e| e.type_id).collect())]
    pub index: Vec<DBPFFileType>,

    #[br(count = item_count)]
    pub entries: Vec<ResourceEntry>,
}
