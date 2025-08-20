// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common::FileName;

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BehaviourConstants {
	pub file_name: FileName,

	#[br(temp)]
	#[bw(calc = constants.len() as u8)]
	count: u8,
	pub flag: u8,
	#[br(count = count)]
	pub constants: Vec<u16>,
}
