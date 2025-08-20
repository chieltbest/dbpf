// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Read, Seek, Write};

use binrw::{
	meta::{EndianKind, ReadEndian, WriteEndian},
	BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt, Endian,
	Endian::Little,
};
#[cfg(test)]
use test_strategy::Arbitrary;

use crate::internal_file::cpf::{cpf_get_all, CPFVersion, Data, Id, Item, Reference, CPF};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct BinaryIndex {
	pub icon: Reference,
	pub stringset: Reference,
	pub bin: Reference,
	pub object: Reference,
	pub creatorid: Id,
	pub sortindex: i32,
	pub stringindex: u32,
}

impl ReadEndian for BinaryIndex {
	const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinRead for BinaryIndex {
	type Args<'a> = ();

	fn read_options<R: Read + Seek>(
		reader: &mut R,
		endian: Endian,
		_args: Self::Args<'_>,
	) -> BinResult<Self> {
		let pos = reader.stream_position().unwrap_or(0);
		let mut cpf: CPF = reader.read_type(endian)?;

		macro_rules! reference {
			($key:ident) => {
				let $key = Reference::read_cpf(&mut cpf, stringify!($key), false, pos)?;
			};
		}

		reference!(icon);
		reference!(stringset);
		reference!(bin);
		reference!(object);

		let sortindex = cpf
			.take_item("sortindex")
			.ok_or(binrw::Error::AssertFail {
				pos,
				message: "Could not find property by key sortindex".to_string(),
			})
			.and_then(|p| match p {
				Data::UInt(i) => Ok(i as i32),
				Data::Int(i) => Ok(i),
				t => Err(binrw::Error::AssertFail {
					pos,
					message: format!("Data of key sortindex has wrong type ({:?})", t.get_type()),
				}),
			})?;

		cpf_get_all!(
			BinaryIndex,
			cpf,
			pos;
			creatorid,
			stringindex;
			icon,
			stringset,
			bin,
			object,
			sortindex
		)
	}
}

impl WriteEndian for BinaryIndex {
	const ENDIAN: EndianKind = EndianKind::Endian(Little);
}

impl BinWrite for BinaryIndex {
	type Args<'a> = ();

	fn write_options<W: Write + Seek>(
		&self,
		writer: &mut W,
		endian: Endian,
		_args: Self::Args<'_>,
	) -> BinResult<()> {
		let BinaryIndex {
			icon,
			stringset,
			bin,
			object,
			creatorid,
			sortindex,
			stringindex,
		} = self;

		macro_rules! get {
			($key:ident) => {
				Item::new(stringify!($key), $key.clone())
			};
		}

		let mut cpf = CPF {
			version: CPFVersion::CPF(2),
			entries: vec![get!(creatorid), get!(sortindex), get!(stringindex)],
		};

		macro_rules! reference {
			($key:ident) => {
				$key.write_cpf(&mut cpf, stringify!($key), false);
			};
		}

		reference!(icon);
		reference!(stringset);
		reference!(bin);
		reference!(object);

		writer.write_type(&cpf, endian)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use proptest::prop_assert_eq;
	use test_strategy::proptest;

	use super::*;

	#[proptest]
	fn write_read_same(binary_index: BinaryIndex) {
		let mut cur = Cursor::new(vec![]);
		binary_index.write(&mut cur)?;
		cur.rewind()?;
		let read = BinaryIndex::read(&mut cur)?;
		prop_assert_eq!(binary_index, read);
	}
}
