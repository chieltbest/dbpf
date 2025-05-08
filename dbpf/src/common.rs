use std::fmt::{Debug, Formatter};
use std::io::{Cursor, Read, Seek, Write};
use std::string::FromUtf8Error;
use binrw::{binrw, BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian};
use derive_more::with_trait::{Deref, DerefMut, Display};
use enum_iterator::Sequence;
#[cfg(test)]
use test_strategy::Arbitrary;

#[binrw]
#[derive(Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct String {
    #[br(temp)]
    #[bw(calc = data.len() as u32)]
    count: u32,
    #[br(count = count)]
    pub data: Vec<u8>,
}

impl Debug for String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", std::string::String::from_utf8_lossy(&self.data))
    }
}

impl<T: AsRef<str>> From<T> for String {
    fn from(value: T) -> Self {
        Self {
            data: Vec::from(value.as_ref()),
        }
    }
}

impl TryFrom<String> for std::string::String {
    type Error = FromUtf8Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        std::string::String::from_utf8(value.data)
    }
}

#[binrw]
#[derive(Clone, Eq, PartialEq, Default, Debug, Display, Deref, DerefMut)]
#[cfg_attr(test, derive(Arbitrary))]
#[deref(forward)]
#[deref_mut(forward)]
pub struct NullString(
    #[cfg_attr(test, map(|x: std::string::String| x.into()))]
    binrw::NullString
);

impl From<&str> for NullString {
    fn from(s: &str) -> Self {
        Self(binrw::NullString::from(s))
    }
}

impl From<std::string::String> for NullString {
    fn from(s: std::string::String) -> Self {
        Self(binrw::NullString::from(s))
    }
}

impl From<NullString> for Vec<u8> {
    fn from(s: NullString) -> Self {
        Vec::<u8>::from(s.0)
    }
}

impl TryFrom<NullString> for std::string::String {
    type Error = FromUtf8Error;

    fn try_from(value: NullString) -> Result<Self, Self::Error> {
        std::string::String::try_from(value.0)
    }
}

#[derive(Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct BigInt {
    pub num: usize,
}

impl BinRead for BigInt {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _endian: Endian, _args: Self::Args<'_>) -> BinResult<Self> {
        let mut offset = 0;
        let mut res = BigInt { num: 0 };
        while {
            let read: u8 = reader.read_ne()?;
            res.num |= (read as usize & 0x7F) << offset;
            offset += 7;
            read & 0x80 != 0
        } {}
        Ok(res)
    }
}

impl BinWrite for BigInt {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(&self, writer: &mut W, _endian: Endian, _args: Self::Args<'_>) -> BinResult<()> {
        let mut num = self.num;
        while {
            writer.write_ne(&(num as u8 & 0x7F))?;
            num >>= 7;
            num > 0
        } {}
        Ok(())
    }
}

/// Also referred to as 7BITSTR, a string that encodes it's length in a variable-length int before the data
#[binrw]
#[derive(Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct BigString {
    #[br(temp)]
    #[bw(calc = BigInt{num: data.len()})]
    len: BigInt,
    #[br(count = len.num)]
    pub data: Vec<u8>,
}

impl Debug for BigString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BigString")
            .field("data", &std::string::String::from_utf8(self.data.clone()))
            .finish()
    }
}

impl From<&str> for BigString {
    fn from(value: &str) -> Self {
        Self {
            data: Vec::from(value)
        }
    }
}

impl From<std::string::String> for BigString {
    fn from(value: std::string::String) -> Self {
        Self {
            data: value.into_bytes()
        }
    }
}

impl TryFrom<BigString> for std::string::String {
    type Error = FromUtf8Error;

    fn try_from(value: BigString) -> Result<Self, Self::Error> {
        std::string::String::from_utf8(value.data)
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct FileName {
    #[brw(pad_size_to = 0x40)]
    #[bw(assert(name.0.len() < 0x40))]
    pub name: NullString,
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug, Default, Sequence)]
#[cfg_attr(test, derive(Arbitrary))]
#[non_exhaustive]
pub enum KnownLanguageCode {
    #[default]
    USEnglish = 1,
    UKEnglish = 2,
    French = 3,
    German = 4,
    Italian = 5,
    Spanish = 6,
    Dutch = 7,
    Danish = 8,
    Swedish = 9,
    Norwegian = 10,
    Finnish = 11,
    Hebrew = 12,
    Russian = 13,
    Portuguese = 14,
    Japanese = 15,
    Polish = 16,
    TraditionalChinese = 17,
    SimplifiedChinese = 18,
    Thai = 19,
    Korean = 20,
    Hindi = 21,
    Arabic = 22,
    Bulgarian = 23,
    Belarusian = 24,
    Ukrainian = 25,
    Czech = 26,
    Greek = 27,
    Hungarian = 28,
    Icelandic = 29,
    Romanian = 30,
    Latin = 31,
    Slovak = 32,
    Albanian = 33,
    Turkish = 34,
    BrazilianPortuguese = 35,
    SwissFrench = 36,
    CanadianFrench = 37,
    BelgianFrench = 38,
    SwissGerman = 39,
    SwissItalian = 40,
    Flemish = 41,
    MexicanSpanish = 42,
    Tagalog = 43,
}

#[binrw]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum LanguageCode {
    Known(KnownLanguageCode),
    Unknown(u8),
}

impl Default for LanguageCode {
    fn default() -> Self {
        LanguageCode::Known(KnownLanguageCode::default())
    }
}

impl From<u8> for LanguageCode {
    fn from(value: u8) -> Self {
        let mut bytes = Cursor::new(vec![value]);
        <Self as BinRead>::read_le(&mut bytes).unwrap()
    }
}
