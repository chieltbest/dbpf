use crate::common::BigString;
use crate::filetypes::{DBPFFileType, KnownDBPFFileType};
use crate::internal_file::resource_collection::geometric_data_container::GeometricDataContainer;
use crate::internal_file::resource_collection::material_definition::MaterialDefinition;
use crate::internal_file::resource_collection::texture_resource::TextureResource;
use binrw::{args, binrw};

pub mod texture_resource;
pub mod material_definition;
pub mod geometric_data_container;

#[binrw]
#[brw(import {type_id: DBPFFileType, version: ResourceBlockVersion})]
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceData {
    #[br(pre_assert(matches ! (type_id, DBPFFileType::Known(KnownDBPFFileType::TextureResource))))]
    Texture(#[brw(args{ version })] TextureResource),
    #[br(pre_assert(matches ! (type_id, DBPFFileType::Known(KnownDBPFFileType::MaterialDefinition))))]
    Material(#[brw(args{ version })] MaterialDefinition),
    #[br(pre_assert(matches ! (type_id, DBPFFileType::Known(KnownDBPFFileType::GeometricDataContainer))))]
    Mesh(#[brw(args{ version })] GeometricDataContainer),
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileName {
    #[br(assert("cSGResource".as_bytes() == c_sg_resource.0.as_slice()))]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum ResourceBlockVersion {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
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
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceEntry {
    pub name: BigString,

    pub type_id: DBPFFileType,
    pub version: ResourceBlockVersion,

    #[brw(args {type_id: type_id.clone(), version: version.clone()})]
    pub data: ResourceData,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, PartialEq)]
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
