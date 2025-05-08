use std::io::{Read, Seek, Write};
use binrw::{BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian};
use binrw::Endian::Little;
use binrw::Error::AssertFail;
use binrw::meta::{EndianKind, ReadEndian, WriteEndian};
use crate::common;
use crate::internal_file::cpf::{cpf_get_all, CPFVersion, Data, Item, Reference, CPF};
use crate::internal_file::cpf::Id;

#[derive(Clone, Debug, Default)]
pub struct Override {
    pub shape: u32,
    pub subset: common::String,

    pub resource: Reference,
}

#[derive(Clone, Debug, Default)]
pub struct PropertySet {
    pub version: Option<u32>,
    pub product: Option<u32>,

    pub parts: u32,
    pub outfit: u32,

    pub priority: Option<u32>,

    pub resource: Reference,
    pub shape: Reference,

    pub age: u32,
    pub gender: u32,
    pub species: u32,
    pub flags: u32,
    pub name: common::String,
    pub creator: Id,
    pub family: Id,
    pub genetic: Option<f32>,
    pub type_: common::String,
    pub skintone: Id,
    pub hairtone: Id,
    pub category: u32,
    pub shoe: u32,
    pub fitness: u32,

    pub overrides: Vec<Override>,
}

impl ReadEndian for PropertySet {
    const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinRead for PropertySet {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, _args: Self::Args<'_>) -> BinResult<Self> {
        let pos = reader.stream_position().unwrap_or(0);
        let mut cpf: CPF = reader.read_type(endian)?;

        macro_rules! get {
            ($key:expr) => {cpf.take_item_verify(pos, $key)};
        }

        let version = get!("version").ok();
        let product = get!("product").ok();

        let parts_res = get!("parts");
        let outfit_res = get!("outfit");

        let (parts, outfit) = match (parts_res, outfit_res) {
            (Ok(p), Ok(o)) => Ok((p, o)),
            (Ok(p), Err(_)) => Ok((p, p)),
            (Err(_), Ok(o)) => Ok((o, o)),
            _ => Err(AssertFail {
                pos,
                message: "Could not find or parse property by key part or outfit".to_string(),
            }),
        }?;

        let resource = Reference::read_cpf(&mut cpf, "resource", true, pos)?;
        let shape = Reference::read_cpf(&mut cpf, "shape", true, pos)?;

        let priority = cpf.take_item("priority").map(|p| {
            match p {
                Data::UInt(i) => Ok(i),
                Data::Int(i) => Ok(i as u32),
                t => Err(AssertFail {
                    pos,
                    message: format!("Data of key priority has wrong type ({:?})", t.get_type()),
                }),
            }
        }).transpose()?;

        let num_overrides: u32 = get!("numoverrides")?;

        let overrides = (0..num_overrides).map(|i| {
            Ok(Override {
                shape: get!(&format!("override{i}shape"))?,
                subset: get!(&format!("override{i}subset"))?,
                resource: Reference::read_cpf(&mut cpf, format!("override{i}resource"), true, pos)?,
            })
        }).collect::<BinResult<_>>()?;

        let genetic = get!("genetic").ok();

        let type_ = get!("type")?;

        cpf_get_all!(
            PropertySet,
            cpf,
            pos;
            age,
            gender,
            species,
            flags,
            name,
            creator,
            family,
            skintone,
            hairtone,
            category,
            shoe,
            fitness;
            version,
            product,
            parts,
            outfit,
            priority,
            resource,
            shape,
            genetic,
            type_,
            overrides
        )
    }
}

impl WriteEndian for PropertySet {
    const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinWrite for PropertySet {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: Endian, _args: Self::Args<'_>) -> BinResult<()> {
        macro_rules! get {
            ($key:ident) => {Item::new(stringify!($key), $key.clone())};
        }

        let PropertySet {
            version,
            product,
            parts,
            outfit,
            priority,
            resource,
            shape,
            age,
            gender,
            species,
            flags,
            name,
            creator,
            family,
            genetic,
            type_,
            skintone,
            hairtone,
            category,
            shoe,
            fitness,
            overrides
        } = self;

        let mut cpf = CPF {
            version: CPFVersion::CPF(2),
            entries: vec![
                get!(parts),
                get!(outfit),
                get!(age),
                get!(gender),
                get!(species),
                get!(flags),
                get!(name),
                get!(creator),
                get!(family),
                Item::new("type", type_.clone()),
                get!(skintone),
                get!(hairtone),
                get!(category),
                get!(shoe),
                get!(fitness),
            ],
        };

        macro_rules! option {
            ($key:ident) => {
                if let Some($key) = $key {
                    cpf.entries.push(Item::new(stringify!($key), *$key));
                }
            };
        }

        option!(version);
        option!(product);
        option!(priority);
        option!(genetic);

        resource.write_cpf(&mut cpf, "resource", true);
        shape.write_cpf(&mut cpf, "shape", true);

        for Override { shape, subset, resource } in overrides {
            cpf.entries.push(get!(shape));
            cpf.entries.push(get!(subset));
            resource.write_cpf(&mut cpf, "resource", true);
        }

        writer.write_type(&cpf, endian)
    }
}
