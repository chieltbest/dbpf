// SPDX-FileCopyrightText: 2026 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::meta::{EndianKind, ReadEndian, WriteEndian};
use binrw::{BinRead, BinResult, BinWrite, Endian};
use std::io::{Read, Seek, Write};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MaterialShader {
	pub text: String,
}

impl ReadEndian for MaterialShader {
	const ENDIAN: EndianKind = EndianKind::None;
}

impl BinRead for MaterialShader {
	type Args<'a> = ();

	fn read_options<R: Read + Seek>(
		reader: &mut R,
		endian: Endian,
		args: Self::Args<'_>,
	) -> BinResult<Self> {
		let mut text = String::new();
		reader.read_to_string(&mut text)?;
		Ok(Self { text })
	}
}

impl WriteEndian for MaterialShader {
	const ENDIAN: EndianKind = EndianKind::None;
}

impl BinWrite for MaterialShader {
	type Args<'a> = ();

	fn write_options<W: Write + Seek>(
		&self,
		writer: &mut W,
		endian: Endian,
		args: Self::Args<'_>,
	) -> BinResult<()> {
		writer.write_all(self.text.as_bytes())?;
		Ok(())
	}
}
