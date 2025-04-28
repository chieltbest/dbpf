use std::fmt::{Debug, Formatter};
use std::io::{Read, Seek, Write};
use std::string::FromUtf8Error;
use binrw::{binrw, BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian, NullString};

#[binrw]
#[derive(Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
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

#[derive(Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
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
pub struct FileName {
    #[brw(pad_size_to = 0x40)]
    #[bw(assert(name.0.len() < 0x40))]
    pub name: NullString,
}

#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug, Default)]
pub enum LanguageCode {
    #[default]
    English = 1,
    Abo = 2,
    Abn = 3,
    Abm = 4,
    Abl = 5,
    Abk = 6,
    Abj = 7,
    Abi = 8,
    Abh = 9,
    Abg = 10,
    Abf = 11,
    Abe = 12,
    Abd = 13,
    Abc = 14,
    Abb = 15,
    Aba = 16,
    Aaz = 17,
    Aay = 18,
    Aax = 19,
    Aaw = 20,
    Aav = 21,
    Aau = 22,
    Aat = 23,
    Aas = 24,
    Aar = 25,
    Aaq = 26,
    Aap = 27,
    Aao = 28,
    Aan = 29,
    Aam = 30,
    Aal = 31,
    Aak = 32,
    Aaj = 33,
    Aai = 34,
    Aah = 35,
    Aag = 36,
    Aaf = 37,
    Aae = 38,
    Aad = 39,
    Aac = 40,
    Aab = 41,
    Aaa = 42,
    Dutch = 43,
}
