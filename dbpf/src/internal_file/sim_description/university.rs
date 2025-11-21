// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::common::Guid;
use binrw::binrw;
use modular_bitfield::bitfield;

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct UniProgressionFlags {
	pub year_1: bool,
	pub year_2: bool,
	pub year_3: bool,
	pub year_4: bool,
	pub good_semester: bool,
	pub probation: bool,
	pub graduated: bool,
	pub at_class: bool,
	pub gates_0: bool,
	pub gates_1: bool,
	pub gates_2: bool,
	pub gates_3: bool,
	pub dropped: bool,
	pub expelled: bool,
	unused_0: bool,
	unused_1: bool,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct UniData {
	pub college_major_guid: Guid,
	pub semester_remaining_time: u16,
	pub progression_flags: UniProgressionFlags,
	pub semester: u16,
	pub on_campus: u16,
	pub influence_bar_level: u16,
	pub influence_minimum: u16,
	pub influence: u16,
}
