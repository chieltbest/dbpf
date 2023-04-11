use binrw::binrw;
use crate::common::String;

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug)]
pub enum DataType {
    UInt = 0xEB61E4F7,
    String = 0x0B8BEA18,
    Float = 0xABC78708,
    Bool = 0xCBA908E1,
    Int = 0x0C264712,
}

#[binrw]
#[br(import {data_type: DataType})]
#[derive(Clone, Debug)]
pub enum Data {
    #[br(pre_assert(matches ! (data_type, DataType::UInt)))]
    UInt(u32),
    #[br(pre_assert(matches ! (data_type, DataType::String)))]
    String(String),
    #[br(pre_assert(matches ! (data_type, DataType::Float)))]
    Float(f32),
    #[br(pre_assert(matches ! (data_type, DataType::Bool)))]
    Bool(u8),
    #[br(pre_assert(matches ! (data_type, DataType::Int)))]
    Int(i32),
}

impl Data {
    fn get_type(&self) -> DataType {
        match self {
            Data::UInt(_) => DataType::UInt,
            Data::String(_) => DataType::String,
            Data::Float(_) => DataType::Float,
            Data::Bool(_) => DataType::Bool,
            Data::Int(_) => DataType::Int,
        }
    }
}

#[binrw]
#[derive(Clone, Debug)]
pub struct Item {
    #[br(temp)]
    #[bw(calc = data.get_type())]
    data_type: DataType,
    pub name: String,
    #[br(args{data_type})]
    pub data: Data,
}

#[binrw]
#[brw(magic = 0xCBE750E0u32)]
#[derive(Clone, Debug)]
pub struct CPF {
    pub version: u16,
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub entries: Vec<Item>,
}
