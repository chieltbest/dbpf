use std::io::{Read, Seek, Write};
use binrw::{BinRead, BinWrite, BinResult, Endian, BinReaderExt, BinWriterExt};
use binrw::Endian::Little;
use binrw::meta::{EndianKind, ReadEndian, WriteEndian};
use crate::internal_file::cpf::{CPF, Item, cpf_get_all, CPFVersion};
use crate::internal_file::cpf::Id;

#[derive(Clone, Debug, Default)]
pub struct BinaryIndex {
    pub iconidx: u32,
    pub stringsetidx: u32,
    pub binidx: u32,
    pub objectidx: u32,
    pub creatorid: Id,
    pub sortindex: i32,
    pub stringindex: u32,
}

impl ReadEndian for BinaryIndex {
    const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinRead for BinaryIndex {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _args: Self::Args<'_>) -> BinResult<Self> {
        let pos = reader.stream_position().unwrap_or(0);
        let cpf: CPF = reader.read_type(endian)?;

        let new = cpf_get_all!(
            BinaryIndex,
            cpf,
            pos;
            iconidx,
            stringsetidx,
            binidx,
            objectidx,
            creatorid,
            sortindex,
            stringindex
        );

        Ok(new)
    }
}

impl WriteEndian for BinaryIndex {
    const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinWrite for BinaryIndex {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: Endian, _args: Self::Args<'_>) -> BinResult<()> {
        macro_rules! get {
            ($key:ident) => {Item::new(stringify!($key), self.$key.clone())};
        }

        let cpf = CPF {
            version: CPFVersion::CPF(2),
            entries: vec![
                get!(iconidx),
                get!(stringsetidx),
                get!(binidx),
                get!(objectidx),
                get!(creatorid),
                get!(sortindex),
                get!(stringindex),
            ],
        };
        writer.write_type(&cpf, endian)
    }
}
