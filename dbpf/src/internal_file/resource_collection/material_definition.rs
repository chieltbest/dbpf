use binrw::binrw;
use crate::common::BigString;
use crate::internal_file::resource_collection::{FileName, ResourceBlockVersion};

#[binrw]
#[derive(Clone, Debug, Default)]
pub struct Property {
    pub name: BigString,
    pub value: BigString,
}

#[binrw]
#[brw(import {version: ResourceBlockVersion})]
#[derive(Clone, Debug, Default)]
pub struct MaterialDefinition {
    pub file_name: FileName,

    pub material_description: BigString,
    pub material_type: BigString,
    
    #[br(temp)]
    #[bw(calc = properties.len() as u32)]
    property_count: u32,
    #[br(count = property_count)]
    pub properties: Vec<Property>,
    
    #[br(temp)]
    #[bw(calc = names.len() as u32)]
    #[brw(if(version as u32 > 8))]
    count: u32,
    #[br(count = count)]
    #[brw(if(version as u32 > 8))]
    pub names: Vec<BigString>,
}