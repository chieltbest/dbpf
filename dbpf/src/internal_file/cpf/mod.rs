use std::{
	fmt::Display,
	io::{Read, Seek, SeekFrom, Write},
	num::{ParseFloatError, ParseIntError},
	str::{FromStr, ParseBoolError},
	string::FromUtf8Error,
};

use binrw::{
	args, binrw,
	meta::{EndianKind, ReadEndian, WriteEndian},
	parser, writer, BinRead, BinResult, BinWrite, Endian, Error,
	Error::AssertFail,
};
use derive_more::{From, TryFrom, TryInto};
#[cfg(test)]
use test_strategy::Arbitrary;
use thiserror::Error;
use xmltree::{Element, ParseError, ParserConfig, XMLNode};

use crate::common::PascalString;

pub type Id = PascalString<u32>;

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
#[cfg_attr(test, derive(Arbitrary))]
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
#[cfg_attr(test, derive(Arbitrary))]
pub enum Data {
	#[br(pre_assert(matches ! (data_type, DataType::UInt)))]
	UInt(u32),
	#[br(pre_assert(matches ! (data_type, DataType::Int)))]
	Int(i32),
	#[br(pre_assert(matches ! (data_type, DataType::String)))]
	String(PascalString<u32>),
	#[br(pre_assert(matches ! (data_type, DataType::Float)))]
	Float(f32),
	#[br(pre_assert(matches ! (data_type, DataType::Bool)))]
	Bool(
		#[br(parse_with = binrw_bool_parser)]
		#[bw(write_with = binrw_bool_writer)]
		bool,
	),
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
#[cfg_attr(test, derive(Arbitrary))]
pub struct Item {
	#[br(temp)]
	#[bw(calc = data.get_type())]
	data_type: DataType,
	pub name: PascalString<u32>,
	#[br(args{data_type})]
	pub data: Data,
}

impl Item {
	pub fn new(name: impl Into<PascalString<u32>>, data: impl Into<Data>) -> Self {
		Self {
			name: name.into(),
			data: data.into(),
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum XMLDataType {
	UInt,
	String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum CPFVersion {
	CPF(u16),
	XML(XMLDataType, Option<u16>),
}

impl Default for CPFVersion {
	fn default() -> Self {
		Self::CPF(2)
	}
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct CPF {
	pub version: CPFVersion,
	pub entries: Vec<Item>,
}

impl ReadEndian for CPF {
	const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

impl BinRead for CPF {
	type Args<'a> = ();

	fn read_options<R: Read + Seek>(
		reader: &mut R,
		_endian: Endian,
		_args: Self::Args<'_>,
	) -> BinResult<Self> {
		let start_pos = reader.stream_position()?;

		if u32::read_le(reader)? == 0xCBE750E0u32 {
			// CPF
			let version = u16::read_le(reader)?;
			let count = u32::read_le(reader)? as usize;
			let entries = Vec::<Item>::read_le_args(reader, args! {
				count
			})?;
			Ok(CPF {
				version: CPFVersion::CPF(version),
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

	fn write_options<W: Write + Seek>(
		&self,
		writer: &mut W,
		_endian: Endian,
		_args: Self::Args<'_>,
	) -> BinResult<()> {
		match self.version {
			CPFVersion::CPF(version) => {
				0xCBE750E0u32.write_le(writer)?;
				version.write_le(writer)?;
				(self.entries.len() as u32).write_le(writer)?;
				self.entries.write_le(writer)
			}
			CPFVersion::XML(data_type, version) => {
				let stream_pos = writer.stream_position()?;
				self.write_xml(writer, data_type, version)
					.map_err(|err| Error::Custom {
						pos: stream_pos,
						err: Box::new(err),
					})
			}
		}
	}
}

#[derive(Debug, Error)]
enum ParsePrimitiveError {
	#[error(transparent)]
	Int(#[from] ParseIntError),
	#[error(transparent)]
	Float(#[from] ParseFloatError),
	#[error(transparent)]
	Bool(#[from] ParseBoolError),
}

#[derive(Debug, Error)]
enum XMLParseError {
	#[error(transparent)]
	Xml(#[from] ParseError),
	#[error("Could not parse version tag: {0:?}")]
	BadVersion(#[from] ParseIntError),
	#[error("Type {name_type:?} does not match type {attribute_type:?}")]
	TypeMismatch {
		name_type: DataType,
		attribute_type: DataType,
	},
	#[error("Start tag is not one of \"cGZPropertySetString\" or \"cGZPropertySetUint32\"")]
	BadStartTag,
	#[error("No key attribute in tag with type {data_type:?}")]
	NoKey { data_type: DataType },
	#[error("Data of item {name:?} with type {data_type:?} could not be parsed: {parse_error}")]
	BadText {
		data_type: DataType,
		name: PascalString<u32>,
		#[source]
		parse_error: ParsePrimitiveError,
	},
}

#[derive(Debug, Error)]
enum XMLKeyUtf8Error {
	#[error(transparent)]
	Utf8(#[from] FromUtf8Error),
	#[error("Cannot have control character {1:?} at position {2} in xml key {0}")]
	Control(String, char, usize),
}

#[derive(Debug, Error)]
enum XMLWriteError {
	#[error(transparent)]
	XMLWrite(#[from] xmltree::Error),
	#[error("{0} while writing key with datatype {1:?}")]
	KeyUtf8(XMLKeyUtf8Error, DataType),
	#[error("{0} while writing data of key {1:?}")]
	DataUtf8(FromUtf8Error, PascalString<u32>),
}

impl CPF {
	fn read_xml<R: Read + Seek>(reader: &mut R) -> Result<Self, XMLParseError> {
		let xml = Element::parse_with_config(
			reader,
			ParserConfig::new()
				.whitespace_to_characters(true)
				.replace_unknown_entity_references(true)
				.add_entity("", ""),
		)?;

		let data_type = match xml.name.as_str() {
			"cGZPropertySetString" => XMLDataType::String,
			"cGZPropertySetUint32" => XMLDataType::UInt,
			_ => return Err(XMLParseError::BadStartTag),
		};
		let version = CPFVersion::XML(
			data_type,
			xml.attributes
				.get("version")
				.map(|str| u16::from_str(str))
				.transpose()?,
		);

		let entries = xml
			.children
			.iter()
			.filter_map(|node| match node {
				XMLNode::Element(e) => {
					macro_rules! parse_int {
						($T:ident, $str:expr) => {
							$str.strip_prefix("0x")
								.map(|hex| $T::from_str_radix(hex, 16))
								.unwrap_or($T::from_str($str))
								.or(i32::from_str($str).map(|i| i as $T))
						};
					}

					let name_type = match e.name.as_str() {
						"AnyUint32" => Some(DataType::UInt),
						"AnySint32" => Some(DataType::Int),
						"AnyString" => Some(DataType::String),
						"AnyFloat32" => Some(DataType::Float),
						"AnyBoolean" => Some(DataType::Bool),
						_ => None,
					};
					let attribute_type = match e
						.attributes
						.get("type")
						.map(|t| {
							parse_int!(u32, t).map(|type_int| {
								DataType::try_from(type_int)
									.expect("u32 datatype can always be created from u32")
							})
						})
						.transpose()
					{
						Err(e) => return Some(Err(e.into())),
						Ok(v) => v,
					};

					let data_type = match (name_type, attribute_type) {
						(Some(t1), Some(t2)) if t1 != t2 => {
							return Some(Err(XMLParseError::TypeMismatch {
								name_type: t1,
								attribute_type: t2,
							}))
						}
						(Some(t), Some(_)) | (Some(t), None) | (None, Some(t)) => Some(t),
						(None, None) => None,
					}?;

					let Some(key) = e.attributes.get("key") else {
						return Some(Err(XMLParseError::NoKey { data_type }));
					};

					let data = match data_type {
						DataType::UInt => parse_int!(u32, &e.get_text().unwrap_or("".into()))
							.map(Data::UInt)
							.map_err(|err| err.into()),
						DataType::Int => parse_int!(i32, &e.get_text().unwrap_or("".into()))
							.map(Data::Int)
							.map_err(|err| err.into()),
						DataType::String => {
							Ok(Data::String(e.get_text().unwrap_or("".into()).into()))
						}
						DataType::Float => f32::from_str(&e.get_text().unwrap_or("".into()))
							.map(Data::Float)
							.map_err(|err| err.into()),
						DataType::Bool => {
							bool::from_str(&e.get_text().unwrap_or("".into()).to_lowercase())
								.map(Data::Bool)
								.map_err(|err| err.into())
						}
					}
					.map_err(|err| XMLParseError::BadText {
						data_type,
						name: key.into(),
						parse_error: err,
					});

					Some(data.map(|data| Item {
						name: key.into(),
						data,
					}))
				}
				_ => None,
			})
			.collect::<Result<Vec<_>, XMLParseError>>()?;

		Ok(CPF { version, entries })
	}

	fn write_xml<W: Write + Seek>(
		&self,
		writer: &mut W,
		data_type: XMLDataType,
		version: Option<u16>,
	) -> Result<(), XMLWriteError> {
		let mut root_element = Element::new(match data_type {
			XMLDataType::UInt => "cGZPropertySetUint32",
			XMLDataType::String => "cGZPropertySetString",
		});

		if let Some(v) = version {
			root_element
				.attributes
				.insert("version".to_string(), v.to_string());
		}

		root_element.children = self
			.entries
			.iter()
			.map(|entry| {
				let data_type = entry.data.get_type();

				let mut element = Element::new(match data_type {
					DataType::UInt => "AnyUint32",
					DataType::String => "AnyString",
					DataType::Float => "AnyFloat32",
					DataType::Bool => "AnyBoolean",
					DataType::Int => "AnySint32",
				});

				element
					.attributes
					.insert("type".to_string(), format!("0x{:x}", data_type as u32));
				element.attributes.insert(
					"key".to_string(),
					entry
						.name
						.clone()
						.try_into()
						.map_err(|err: FromUtf8Error| XMLWriteError::KeyUtf8(err.into(), data_type))
						.and_then(|str: String| {
							if let Some((i, c)) =
								str.chars().enumerate().find(|(i, c)| *c <= '\x1F')
							{
								Err(XMLWriteError::KeyUtf8(
									XMLKeyUtf8Error::Control(str, c, i),
									data_type,
								))
							} else {
								Ok(str)
							}
						})?,
				);

				element.children.push(XMLNode::Text(match &entry.data {
					Data::UInt(x) => x.to_string(),
					Data::Int(x) => x.to_string(),
					Data::Float(x) => x.to_string(),
					Data::String(str) => str
						.clone()
						.try_into()
						.map_err(|err| XMLWriteError::DataUtf8(err, entry.name.clone()))?,
					Data::Bool(b) => if *b { "True" } else { "False" }.to_string(),
				}));

				Ok(XMLNode::Element(element))
			})
			.collect::<Result<Vec<_>, XMLWriteError>>()?;

		root_element.write(writer).map_err(|err| err.into())
	}

	pub fn take_item(&mut self, key: &str) -> Option<Data> {
		let ret = self
			.entries
			.iter()
			.find_map(|item| (item.name == key.into()).then_some(item.data.clone()));
		self.entries.retain(|item| item.name != key.into());
		ret
	}

	pub fn take_item_verify<T>(&mut self, stream_position: u64, key: &str) -> BinResult<T>
	where
		T: TryFrom<Data>,
		<T as TryFrom<Data>>::Error: Display,
	{
		self.take_item(key)
			.ok_or(AssertFail {
				pos: stream_position,
				message: format!("Could not find property by key {key}"),
			})
			.and_then(|data| {
				data.try_into().map_err(|err| AssertFail {
					pos: stream_position,
					message: format!("Data of key {key} has wrong type ({})", err),
				})
			})
	}

	fn check_empty(&self, pos: u64) -> BinResult<()> {
		if !self.entries.is_empty() {
			Err(AssertFail {
				pos,
				message: format!("CPF still has remaining entries: {:?}", self.entries),
			})
		} else {
			Ok(())
		}
	}
}

macro_rules! cpf_get_all {
    ($t:ident, $cpf:expr, $pos:expr; $($keys:ident),*; $($extras:ident),*) => {
        {
            let ret = $t {
                $(
                    $keys: $cpf.take_item_verify($pos, stringify!($keys))?,
                )*
                $(
                    $extras,
                )*
            };

            $cpf.check_empty($pos).map(|_| ret)
        }
    };
    ($t:ident, $cpf:expr, $pos:expr; $($keys:ident),*) => {
        {
            let ret = $t {
                $(
                    $keys: $cpf.take_item_verify($pos, stringify!($keys))?,
                )*
            };

            $cpf.check_empty($pos).map(|_| ret)
        }
    };
}
pub(crate) use cpf_get_all;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Reference {
	Idx(u32),
	TGI(u32, u32, u32),
}

impl Default for Reference {
	fn default() -> Self {
		Self::Idx(0)
	}
}

impl Reference {
	fn read_cpf<S: AsRef<str>>(
		cpf: &mut CPF,
		name: S,
		key: bool,
		stream_position: u64,
	) -> BinResult<Self> {
		let idx = if key { "keyidx" } else { "idx" };
		if let Ok(idx) = cpf.take_item_verify(stream_position, &format!("{}{idx}", name.as_ref())) {
			Ok(Self::Idx(idx))
		} else {
			Ok(Self::TGI(
				cpf.take_item_verify(stream_position, &format!("{}restypeid", name.as_ref()))?,
				cpf.take_item_verify(stream_position, &format!("{}groupid", name.as_ref()))?,
				cpf.take_item_verify(stream_position, &format!("{}id", name.as_ref()))?,
			))
		}
	}

	fn write_cpf<S: AsRef<str>>(&self, cpf: &mut CPF, name: S, key: bool) {
		let keyidx = if key { "keyidx" } else { "idx" };
		match self {
			Reference::Idx(idx) => {
				cpf.entries
					.push(Item::new(format!("{}{keyidx}", name.as_ref()), *idx));
			}
			Reference::TGI(t, g, i) => {
				cpf.entries
					.push(Item::new(format!("{}restypeid", name.as_ref()), *t));
				cpf.entries
					.push(Item::new(format!("{}groupid", name.as_ref()), *g));
				cpf.entries
					.push(Item::new(format!("{}id", name.as_ref()), *i));
			}
		}
	}
}

#[cfg(test)]
mod test {
	use std::io::{Cursor, Seek};

	use binrw::{BinRead, BinWrite};
	use proptest::prop_assert_eq;
	use test_strategy::proptest;

	use crate::{
		common::PascalString,
		internal_file::cpf::{CPFVersion, Data, Item, XMLDataType, CPF},
	};

	#[proptest]
	fn binary_write_read_same(version: u16, entries: Vec<Item>) {
		let cpf = CPF {
			version: CPFVersion::CPF(version),
			entries,
		};
		let mut out = Cursor::new(vec![]);
		cpf.write(&mut out)?;
		out.rewind()?;
		let read = CPF::read(&mut out)?;
		prop_assert_eq!(cpf, read);
	}

	/// writing of a XML CPF object that has no invalid strings must not fail
	#[proptest]
	fn xml_write_restricted_no_error(
		data_type: XMLDataType,
		version: Option<u16>,
		entries: Vec<Item>,
	) {
		fn str_is_ok(str: PascalString<u32>) -> bool {
			std::string::String::try_from(str).is_ok_and(|str| str.chars().all(|c| !c.is_control()))
		}
		for e in &entries {
			if !str_is_ok(e.name.clone()) {
				return Ok(());
			}
			if let Data::String(str) = e.data.clone() {
				if !str_is_ok(str) {
					return Ok(());
				}
			}
		}
		let cpf = CPF {
			version: CPFVersion::XML(data_type, version),
			entries,
		};
		let mut out = Cursor::new(vec![]);
		cpf.write(&mut out)?
	}

	/// a succesful write of a XML CPF object must also be able to be read correctly
	#[proptest]
	fn xml_write_no_error_read_same(
		data_type: XMLDataType,
		version: Option<u16>,
		entries: Vec<Item>,
	) {
		let cpf = CPF {
			version: CPFVersion::XML(data_type, version),
			entries,
		};
		let mut out = Cursor::new(vec![]);
		if cpf.write(&mut out).is_ok() {
			out.rewind()?;
			let read = CPF::read(&mut out)?;
			prop_assert_eq!(cpf, read);
		}
	}
}
