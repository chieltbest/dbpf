// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common::FileName;

#[binrw]
#[brw(magic = b"fJBO")]
struct HeaderMagic;

#[binrw]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Header {
	Normal(
		[u8; 0x8],
		#[br(temp)]
		#[bw(calc = HeaderMagic)]
		HeaderMagic,
	),
	ExtraNull(
		[u8; 0x48],
		#[br(temp)]
		#[bw(calc = HeaderMagic)]
		HeaderMagic,
	),
}

impl Default for Header {
	fn default() -> Self {
		Header::Normal([0; 0x8])
	}
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Entry {
	pub guardian_id: u16,
	pub action_id: u16,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectFunctions {
	pub file_name: FileName,
	pub header: Header,

	#[br(temp)]
	#[bw(calc = entries.len() as u32)]
	count: u32,
	#[br(count = count)]
	pub entries: Vec<Entry>,
}
