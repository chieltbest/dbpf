use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::fmt::{Debug, Formatter};
use std::io::{Read, Seek, Write};
use std::string::FromUtf8Error;
use binrw::{BinRead, BinReaderExt, BinResult, binrw, BinWrite, BinWriterExt, Endian};

#[binrw]
#[derive(Clone, Default)]
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

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        Self {
            data: value.into_bytes()
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
            .field("data", &OsString::from_vec(self.data.clone()))
            .finish()
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
