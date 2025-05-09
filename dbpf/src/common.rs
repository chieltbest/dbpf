use std::fmt::{Debug, Formatter};
use std::io::{Cursor, Read, Seek, Write};
use std::marker::PhantomData;
use std::string::FromUtf8Error;
use binrw::{args, binrw, BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian, NamedArgs};
use binrw::error::CustomError;
use binrw::meta::{EndianKind, ReadEndian, WriteEndian};
use derive_more::with_trait::{Deref, DerefMut};
use enum_iterator::Sequence;
#[cfg(test)]
use test_strategy::Arbitrary;

/// common string type without binread/binwrite implementation
#[derive(Clone, Default, Hash, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ByteString(Vec<u8>);

impl<T> From<PascalString<T>> for ByteString {
    fn from(value: PascalString<T>) -> Self {
        Self(value.data)
    }
}

impl From<NullString> for ByteString {
    fn from(value: NullString) -> Self {
        Self(value.0)
    }
}

impl TryInto<String> for ByteString {
    type Error = FromUtf8Error;

    fn try_into(self) -> Result<String, Self::Error> {
        String::from_utf8(self.0)
    }
}

impl From<String> for ByteString {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}

impl Debug for ByteString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", String::from_utf8_lossy(&self.0))
    }
}

#[derive(Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct PascalString<T> {
    _t: PhantomData<T>,
    pub data: Vec<u8>,
}

impl<T> ReadEndian for PascalString<T> {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

impl<T: TryInto<usize> + BinRead> BinRead for PascalString<T>
where
    <T as TryInto<usize>>::Error: CustomError + 'static,
{
    type Args<'a> = <T as BinRead>::Args<'a>;

    fn read_options<R: Read + Seek>(reader: &mut R, _endian: Endian, args: Self::Args<'_>) -> BinResult<Self> {
        let pos = reader.stream_position()?;
        let count = T::read_le_args(reader, args)?.try_into()
            .map_err(|err| binrw::Error::Custom {
                pos,
                err: Box::new(err),
            })?;
        Ok(Self {
            _t: PhantomData,
            data: Vec::<u8>::read_le_args(reader, args! { count })?,
        })
    }
}

impl<T> WriteEndian for PascalString<T> {
    const ENDIAN: EndianKind = EndianKind::Endian(Endian::Little);
}

impl<T: TryFrom<usize> + BinWrite> BinWrite for PascalString<T>
where
    <T as TryFrom<usize>>::Error: CustomError + 'static,
{
    type Args<'a> = <T as BinWrite>::Args<'a>;

    fn write_options<W: Write + Seek>(&self, writer: &mut W, _endian: Endian, args: Self::Args<'_>) -> BinResult<()> {
        let pos = writer.stream_position()?;
        let count: T = self.data.len().try_into()
            .map_err(|err| binrw::Error::Custom {
                pos,
                err: Box::new(err),
            })?;
        count.write_le_args(writer, args)?;
        self.data.write(writer)
    }
}

impl<T> Debug for PascalString<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", String::from_utf8_lossy(&self.data))
    }
}

impl<T: AsRef<str>, N> From<T> for PascalString<N> {
    fn from(value: T) -> Self {
        Self {
            _t: PhantomData,
            data: Vec::from(value.as_ref()),
        }
    }
}

impl<T> TryFrom<PascalString<T>> for String {
    type Error = FromUtf8Error;

    fn try_from(value: PascalString<T>) -> Result<Self, Self::Error> {
        String::from_utf8(value.data)
    }
}

impl<T> From<ByteString> for PascalString<T> {
    fn from(value: ByteString) -> Self {
        Self {
            _t: PhantomData,
            data: value.0,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Default, Deref, DerefMut)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct NullString(
    Vec<u8>,
);

#[derive(NamedArgs, Default)]
pub struct NullStringArgs {
    count: Option<usize>,
}

impl ReadEndian for NullString {
    const ENDIAN: EndianKind = EndianKind::None;
}

impl BinRead for NullString {
    type Args<'a> = NullStringArgs;

    fn read_options<R: Read + Seek>(reader: &mut R, _endian: Endian, args: Self::Args<'_>) -> BinResult<Self> {
        let mut out = vec![];
        let mut cur_byte = u8::read_ne(reader)?;
        let mut count = 1usize;
        while cur_byte != 0 {
            out.push(cur_byte);
            if args.count.is_some_and(|c| count >= c) {
                break;
            }
            cur_byte = u8::read_ne(reader)?;
            count += 1;
        }
        if let Some(c) = args.count {
            reader.seek_relative((c - count) as i64)?;
        }
        Ok(Self(out))
    }
}

impl WriteEndian for NullString {
    const ENDIAN: EndianKind = EndianKind::None;
}

impl BinWrite for NullString {
    type Args<'a> = NullStringArgs;

    fn write_options<W: Write + Seek>(&self, writer: &mut W, _endian: Endian, args: Self::Args<'_>) -> BinResult<()> {
        if let Some(c) = args.count {
            if self.0.len() > c {
                return Err(binrw::Error::AssertFail {
                    pos: writer.stream_position()?,
                    message: format!("Number of bytes is greater than maximum allowed: {} > {c}", self.0.len()),
                });
            }
        }

        self.0.write(writer)?;

        if let Some(c) = args.count {
            [0u8].repeat(c - self.0.len()).write(writer)
        } else {
            0u8.write(writer)
        }
    }
}

impl Debug for NullString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", String::from_utf8_lossy(&self.0))
    }
}

impl From<Vec<u8>> for NullString {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&str> for NullString {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for NullString {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<NullString> for Vec<u8> {
    fn from(s: NullString) -> Self {
        s.0
    }
}

impl TryFrom<NullString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: NullString) -> Result<Self, Self::Error> {
        String::try_from(value.0)
    }
}

impl From<ByteString> for NullString {
    fn from(value: ByteString) -> Self {
        Self(value.0)
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
            let cur_write_byte = num as u8 & 0x7F;
            num >>= 7;
            let has_more = num > 0;
            writer.write_ne(&(cur_write_byte | if has_more { 0x80 } else { 0 }))?;
            has_more
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
            .field("data", &String::from_utf8(self.data.clone()))
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

impl From<String> for BigString {
    fn from(value: String) -> Self {
        Self {
            data: value.into_bytes()
        }
    }
}

impl TryFrom<BigString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: BigString) -> Result<Self, Self::Error> {
        String::from_utf8(value.data)
    }
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct FileName {
    #[brw(args { count: Some(0x40) })]
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

#[cfg(test)]
mod test {
    use test_strategy::proptest;
    use crate::common::PascalString;

    #[proptest]
    #[should_panic]
    fn string_sometimes_invalid_utf8(string: PascalString<u32>) {
        std::string::String::try_from(string)?;
    }
}
