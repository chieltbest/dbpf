use std::fmt::{Debug, Formatter};
use std::fs::read;
use std::io::{Read, Seek, Write};
use binrw::{BinRead, BinReaderExt, BinResult, binrw, BinWrite, BinWriterExt, Endian};

#[binrw]
#[derive(Clone)]
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

pub struct BigInt {
    num: usize,
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
pub struct BigString {
    #[bw(calc = BigInt{num: data.len()})]
    len: BigInt,
    #[br(count = len.num)]
    data: Vec<u8>,
}
