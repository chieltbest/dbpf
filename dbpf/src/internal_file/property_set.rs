use std::io::{Read, Seek, Write};
use binrw::{BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian};
use binrw::Endian::Little;
use binrw::meta::{EndianKind, ReadEndian, WriteEndian};
use crate::common;
use crate::internal_file::common::cpf::{cpf_get_all, Item, CPF};
use crate::internal_file::common::Id;
// use crate::internal_file::xml::XML;

#[derive(Clone, Debug, Default)]
pub struct Override {
    pub shape: u32,
    pub subset: common::String,
    pub resourcekeyidx: u32,
}

#[derive(Clone, Debug, Default)]
pub struct PropertySet {
    pub version: u32,
    pub product: u32,
    pub age: u32,
    pub gender: u32,
    pub species: u32,
    pub parts: u32,
    pub outfit: u32,
    pub flags: u32,
    pub name: common::String,
    pub creator: Id,
    pub family: Id,
    pub genetic: f32,
    pub priority: u32,
    pub type_: common::String,
    pub skintone: Id,
    pub hairtone: Id,
    pub category: u32,
    pub shoe: u32,
    pub fitness: u32,
    pub resourcekeyidx: u32,
    pub shapekeyidx: u32,

    pub overrides: Vec<Override>,
}

impl ReadEndian for PropertySet {
    const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinRead for PropertySet {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _args: Self::Args<'_>) -> BinResult<Self> {
        let pos = reader.stream_position().unwrap_or(0);
        let cpf: CPF = reader.read_type(endian)?;

        macro_rules! get {
            ($key:expr) => {cpf.get_item_verify(pos, $key)};
        }

        let num_overrides: u32 = get!("numoverrides")?;

        let overrides = (0..num_overrides).map(|i| {
            Ok(Override {
                shape: get!(&format!("override{i}shape"))?,
                subset: get!(&format!("override{i}subset"))?,
                resourcekeyidx: get!(&format!("override{i}resourcekeyidx"))?,
            })
        }).collect::<BinResult<_>>()?;

        let priority = get!("priority")
            .or_else(|_| cpf.get_item_verify::<i32>(pos, "priority")
                .map(|n| n as u32))?;

        let type_ = get!("type")?;

        let new = cpf_get_all!(
            PropertySet,
            cpf,
            pos;
            version,
            product,
            age,
            gender,
            species,
            parts,
            outfit,
            flags,
            name,
            creator,
            family,
            genetic,
            skintone,
            hairtone,
            category,
            shoe,
            fitness,
            resourcekeyidx,
            shapekeyidx;
            priority,
            type_,
            overrides
        );

        Ok(new)
    }
}

impl WriteEndian for PropertySet {
    const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinWrite for PropertySet {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: Endian, _args: Self::Args<'_>) -> BinResult<()> {
        macro_rules! get {
            ($key:ident) => {Item::new(stringify!($key), self.$key.clone())};
        }

        let mut entries = vec![
            get!(version),
            get!(product),
            get!(age),
            get!(gender),
            get!(species),
            get!(parts),
            get!(outfit),
            get!(flags),
            get!(name),
            get!(creator),
            get!(family),
            get!(genetic),
            get!(priority),
            Item::new("type", self.type_.clone()),
            get!(skintone),
            get!(hairtone),
            get!(category),
            get!(shoe),
            get!(fitness),
            get!(resourcekeyidx),
            get!(shapekeyidx),
            Item::new("numoverrides", self.overrides.len() as u32),
        ];

        for (i, o) in self.overrides.iter().enumerate() {
            entries.push(Item::new(format!("override{i}shape"), o.shape));
            entries.push(Item::new(format!("override{i}subset"), o.subset.clone()));
            entries.push(Item::new(format!("override{i}resourcekeyidx"), o.resourcekeyidx));
        }

        let cpf = CPF {
            version: 2,
            entries,
        };
        writer.write_type(&cpf, endian)
    }
}
