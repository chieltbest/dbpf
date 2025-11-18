use crate::common::Guid;
use binrw::binrw;
use modular_bitfield::bitfield;

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UniProgressionFlags {
	pub year_1: bool,
	pub year_2: bool,
	pub year_3: bool,
	pub year_4: bool,
	pub semester: bool,
	pub try_period: bool,
	pub got_diploma: bool,
	pub in_course_or_exam: bool,
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UniData {
	pub uni_college_major_guid: Guid,
	pub uni_semester_remaining_time: u16,
	pub uni_progression_flags: UniProgressionFlags,
	pub uni_semester: u16,
	pub uni_on_campus: u16,
	pub uni_influence_bar_level: u16,
	pub uni_influence_minimum: u16,
	pub uni_influence: u16,
}
