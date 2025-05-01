use crate::common::String;
use binrw::meta::{EndianKind, ReadEndian, WriteEndian};
use binrw::Error::AssertFail;
use binrw::{args, binrw, parser, writer, BinRead, BinResult, BinWrite, Endian, Error};
use derive_more::{From, TryFrom, TryInto};
use std::fmt::Display;
use std::io::{Read, Seek, SeekFrom, Write};
use std::num::ParseIntError;
use std::str::FromStr;
use std::string::FromUtf8Error;
use thiserror::Error;
use xmltree::{Element, ParseError, ParserConfig, XMLNode};

pub type Id = String;

pub mod binary_index;
pub mod property_set;

#[parser(reader)]
fn binrw_bool_parser() -> BinResult<bool> {
    u8::read(reader).map(|u| u > 0)
}

#[writer(writer)]
fn binrw_bool_writer(b: &bool) -> BinResult<()> {
    (*b as u8).write(writer)
}

#[binrw]
#[brw(repr = u32)]
#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFrom)]
#[try_from(repr)]
pub enum DataType {
    UInt = 0xEB61E4F7,
    String = 0x0B8BEA18,
    Float = 0xABC78708,
    Bool = 0xCBA908E1,
    Int = 0x0C264712,
}

#[binrw]
#[br(import{data_type: DataType})]
#[derive(Clone, Debug, PartialEq, From, TryInto)]
pub enum Data {
    #[br(pre_assert(matches ! (data_type, DataType::UInt)))]
    UInt(u32),
    #[br(pre_assert(matches ! (data_type, DataType::Int)))]
    Int(i32),
    #[br(pre_assert(matches ! (data_type, DataType::String)))]
    String(String),
    #[br(pre_assert(matches ! (data_type, DataType::Float)))]
    Float(f32),
    #[br(pre_assert(matches ! (data_type, DataType::Bool)))]
    Bool(#[br(parse_with = binrw_bool_parser)]
         #[bw(write_with = binrw_bool_writer)]
         bool),
}

impl Data {
    pub fn get_type(&self) -> DataType {
        match self {
            Data::UInt(_) => DataType::UInt,
            Data::String(_) => DataType::String,
            Data::Float(_) => DataType::Float,
            Data::Bool(_) => DataType::Bool,
            Data::Int(_) => DataType::Int,
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self::UInt(0)
    }
}

#[binrw]
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Item {
    #[br(temp)]
    #[bw(calc = data.get_type())]
    data_type: DataType,
    pub name: String,
    #[br(args{data_type})]
    pub data: Data,
}

impl Item {
    pub fn new(name: impl Into<String>, data: impl Into<Data>) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
        }
    }
}

#[binrw]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CPFVersion {
    CPF(u16),
    XML(DataType, Option<u16>),
}

impl Default for CPFVersion {
    fn default() -> Self {
        Self::CPF(2)
    }
}

#[derive(Clone, Debug)]
pub struct CPF {
    pub version: CPFVersion,
    pub entries: Vec<Item>,
}

impl ReadEndian for CPF {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

impl BinRead for CPF {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _endian: Endian, _args: Self::Args<'_>) -> BinResult<Self> {
        let start_pos = reader.stream_position()?;

        if u32::read_le(reader)? == 0xCBE750E0u32 {
            // CPF
            let version = CPFVersion::read_le(reader)?;
            let count = u32::read_le(reader)? as usize;
            let entries = Vec::<Item>::read_le_args(reader, args! {
                count
            })?;
            Ok(CPF {
                version,
                entries,
            })
        } else {
            // XML
            reader.seek(SeekFrom::Start(start_pos))?;
            CPF::read_xml(reader).map_err(|err| AssertFail {
                pos: start_pos,
                message: err.to_string(),
            })
        }
    }
}

impl WriteEndian for CPF {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

impl BinWrite for CPF {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(&self, writer: &mut W, _endian: Endian, _args: Self::Args<'_>) -> BinResult<()> {
        match self.version {
            CPFVersion::CPF(version) => {
                0xCBE750E0u32.write_le(writer)?;
                version.write_le(writer)?;
                (self.entries.len() as u32).write_le(writer)?;
                self.entries.write_le(writer)
            }
            CPFVersion::XML(data_type, version) => {
                let stream_pos = writer.stream_position()?;
                self.write_xml(writer, data_type, version).map_err(|err|
                    AssertFail {
                        pos: stream_pos,
                        message: err.to_string(),
                    })
            }
        }
    }
}

#[derive(Debug, Error)]
enum XMLParseError {
    #[error(transparent)]
    XML(#[from] ParseError),
    #[error("Could not parse version number: {0}")]
    VersionParseError(#[from] ParseIntError),
    #[error("Type {name_type:?} does not match type {attribute_type:?}")]
    TypeMismatch {
        name_type: DataType,
        attribute_type: DataType,
    },
    #[error("Start tag is not one of \"cGZPropertySetString\" or \"cGZPropertySetUint32\"")]
    BadStartTag,
    #[error("No key attribute in tag with type {data_type:?}")]
    NoKey {
        data_type: DataType,
    },
    #[error("Data of item {data_type:?} with type {name:?} could not be parsed: {parse_error}")]
    BadText {
        data_type: DataType,
        name: String,
        parse_error: std::string::String,
    },
}

#[derive(Debug, Error)]
enum XMLWriteError {
    #[error(transparent)]
    XMLWriteError(#[from] xmltree::Error),
    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Bad header data type: {0:?}")]
    HeaderDataType(DataType),
}

impl CPF {
    fn read_xml<R: Read + Seek>(reader: &mut R) -> Result<Self, XMLParseError> {
        let xml = Element::parse_with_config(
            reader,
            ParserConfig::new()
                .whitespace_to_characters(true)
                .replace_unknown_entity_references(true)
                .add_entity("", ""))?;

        let data_type = match xml.name.as_str() {
            "cGZPropertySetString" => {
                DataType::String
            }
            "cGZPropertySetUint32" => {
                DataType::UInt
            }
            _ => return Err(XMLParseError::BadStartTag),
        };
        let version = CPFVersion::XML(
            data_type,
            xml.attributes.get("version")
                .map(|str| u16::from_str(str)).transpose()?);

        let entries = xml.children.iter().filter_map(|node| {
            match node {
                XMLNode::Element(e) => {
                    macro_rules! parse_int {
                        ($T:ident, $str:expr) => {
                            $str.strip_prefix("0x")
                                .and_then(|hex| $T::from_str_radix(hex, 16).ok())
                                .or_else(|| $T::from_str($str).ok())
                        };
                    }

                    let name_type = match e.name.as_str() {
                        "AnyUint32" => Some(DataType::UInt),
                        "AnySint32" => Some(DataType::Int),
                        "AnyString" => Some(DataType::String),
                        "AnyFloat32" => Some(DataType::Float),
                        "AnyBoolean" => Some(DataType::Bool),
                        _ => None
                    };
                    let attribute_type = e.attributes.get("type")
                        .and_then(|t| {
                            parse_int!(u32, t)
                                .and_then(|type_int| DataType::try_from(type_int).ok())
                        });

                    let data_type = match (name_type, attribute_type) {
                        (Some(t1), Some(t2))
                        if t1 != t2 => return Some(Err(XMLParseError::TypeMismatch {
                            name_type: t1,
                            attribute_type: t2,
                        })),
                        (Some(t), Some(_))
                        | (Some(t), None)
                        | (None, Some(t)) => Some(t),
                        (None, None) => None,
                    }?;

                    let Some(key) = e.attributes.get("key")
                    else {
                        return Some(Err(XMLParseError::NoKey {
                            data_type
                        }))
                    };

                    let data = match data_type {
                        DataType::UInt => e.get_text()
                            .and_then(|str| parse_int!(u32, &str))
                            .map(|i| Data::UInt(i)),
                        DataType::String => Some(Data::String(e.get_text().unwrap_or("".into()).into())),
                        DataType::Float => e.get_text()
                            .and_then(|str| f32::from_str(&str).ok())
                            .map(|i| Data::Float(i)),
                        DataType::Bool => e.get_text()
                            .and_then(|str| bool::from_str(&str.to_lowercase()).ok())
                            .map(|i| Data::Bool(i)),
                        DataType::Int => e.get_text()
                            .and_then(|str| parse_int!(i32, &str))
                            .map(|i| Data::Int(i)),
                    };

                    data.map(|data| {
                        Ok(Item {
                            name: key.into(),
                            data,
                        })
                    })
                }
                _ => None
            }
        }).collect::<Result<Vec<_>, XMLParseError>>()?;

        Ok(CPF {
            version,
            entries,
        })
    }

    fn write_xml<W: Write + Seek>(&self, writer: &mut W, data_type: DataType, version: Option<u16>) -> Result<(), XMLWriteError> {
        let mut root_element = Element::new(match data_type {
            DataType::UInt => "cGZPropertySetUint32",
            DataType::String => "cGZPropertySetString",
            t => return Err(XMLWriteError::HeaderDataType(t)),
        });

        if let Some(v) = version {
            root_element.attributes.insert("version".to_string(), v.to_string());
        }

        root_element.children = self.entries.iter().map(|entry| {
            let data_type = entry.data.get_type();

            let mut element = Element::new(match data_type {
                DataType::UInt => "AnyUint32",
                DataType::String => "AnyString",
                DataType::Float => "AnyFloat32",
                DataType::Bool => "AnyBoolean",
                DataType::Int => "AnyInt",
            });

            element.attributes.insert("type".to_string(), format!("0x{:x}", data_type as u32));
            element.attributes.insert("key".to_string(), entry.name.clone().try_into()?);

            element.children.push(XMLNode::Text(match &entry.data {
                Data::UInt(x) => x.to_string(),
                Data::Int(x) => x.to_string(),
                Data::Float(x) => x.to_string(),
                Data::String(str) => str.clone().try_into()?,
                Data::Bool(b) => if *b { "True" } else { "False" }.to_string(),
            }));

            Ok(XMLNode::Element(element))
        }).collect::<Result<Vec<_>, XMLWriteError>>()?;

        root_element.write(writer).map_err(|err| err.into())
    }

    pub fn get_item(&self, key: &str) -> Option<&Data> {
        self.entries.iter().find_map(|item| {
            (item.name == key.into()).then_some(&item.data)
        })
    }

    pub fn get_item_verify<T>(&self, stream_position: u64, key: &str) -> BinResult<T>
    where
        T: TryFrom<Data>,
        <T as TryFrom<Data>>::Error: Display,
    {
        self.get_item(key).ok_or(Error::AssertFail {
            pos: stream_position,
            message: format!("Could not find property by key {key}"),
        }).and_then(|data| {
            data.clone().try_into()
                .map_err(|err| Error::AssertFail {
                    pos: stream_position,
                    message: format!("Data of key {key} has wrong type ({})", err),
                })
        })
    }
}

macro_rules! cpf_get_all {
    ($t:ident, $cpf:expr, $pos:expr; $($keys:ident),*; $($extras:ident),*) => {
        $t {
            $(
                $keys: $cpf.get_item_verify($pos, stringify!($keys))?,
            )*
            $(
                $extras,
            )*
        }
    };
    ($t:ident, $cpf:expr, $pos:expr; $($keys:ident),*) => {
        $t {
            $(
                $keys: $cpf.get_item_verify($pos, stringify!($keys))?,
            )*
        }
    };
}
pub(crate) use cpf_get_all;
