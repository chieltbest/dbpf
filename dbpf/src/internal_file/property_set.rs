use std::collections::HashMap;
use std::fmt::Display;
use std::io::{Read, Seek, Write};
use std::string::FromUtf8Error;
use binrw::{BinRead, BinWrite, BinResult, Endian, BinReaderExt, Error, BinWriterExt};
use binrw::Endian::Little;
use binrw::meta::{EndianKind, ReadEndian, WriteEndian};
use crate::common;
use crate::internal_file::common::cpf::{CPF, Data, Item};
// use crate::internal_file::xml::XML;

type Id = common::String;

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

        let mut entries: HashMap<String, Data> = cpf.entries
            .into_iter()
            .map(|entry| {
                Ok((entry.name.try_into()
                        .map_err(|err: FromUtf8Error| Error::AssertFail {
                            pos,
                            message: err.to_string(),
                        })?,
                    entry.data))
            })
            .collect::<BinResult<Vec<(String, Data)>>>()?
            .into_iter()
            .collect();

        fn get_fn<T>(entries: &mut HashMap<String, Data>, pos: u64, key: &str) -> BinResult<T>
        where
            T: TryFrom<Data>,
            <T as TryFrom<Data>>::Error: Display,
        {
            entries.get(key).ok_or(Error::AssertFail {
                pos,
                message: format!("Could not find property by key {key}"),
            }).and_then(|data| {
                data.clone().try_into()
                    .map_err(|err| Error::AssertFail {
                        pos,
                        message: format!("Data of key {key} has wrong type ({})", err),
                    })
            })
        }

        macro_rules! get {
            ($key:expr) => {get_fn(&mut entries, pos, $key)};
        }

        let num_overrides: u32 = get!("numoverrides")?;

        let overrides = (0..num_overrides).map(|i| {
            Ok(Override {
                shape: get!(&format!("override{i}shape"))?,
                subset: get!(&format!("override{i}subset"))?,
                resourcekeyidx: get!(&format!("override{i}resourcekeyidx"))?,
            })
        }).collect::<BinResult<_>>()?;

        let new = PropertySet {
            version: get!("version")?,
            product: get!("product")?,
            age: get!("age")?,
            gender: get!("gender")?,
            species: get!("species")?,
            parts: get!("parts")?,
            outfit: get!("outfit")?,
            flags: get!("flags")?,
            name: get!("name")?,
            creator: get!("creator")?,
            family: get!("family")?,
            genetic: get!("genetic")?,
            priority: get!("priority")
                .or_else(|_| get_fn::<i32>(&mut entries,
                                           pos,
                                           "priority")
                    .map(|n| n as u32))?,
            type_: get!("type")?,
            skintone: get!("skintone")?,
            hairtone: get!("hairtone")?,
            category: get!("category")?,
            shoe: get!("shoe")?,
            fitness: get!("fitness")?,
            resourcekeyidx: get!("resourcekeyidx")?,
            shapekeyidx: get!("shapekeyidx")?,
            overrides,
        };

        Ok(new)
    }
}

impl WriteEndian for PropertySet {
    const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinWrite for PropertySet {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: Endian, _args: Self::Args<'_>) -> BinResult<()> {
        // TODO use macro to prevent silly bugs :)

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
