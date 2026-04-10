// SPDX-FileCopyrightText: 2026 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;
use binrw::helpers::until_eof;

#[binrw]
#[brw(little)]
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Image {
	#[br(parse_with = until_eof)]
	pub data: Vec<u8>,
}
