// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;
use enum_iterator::Sequence;
use modular_bitfield::bitfield;

#[binrw]
#[brw(repr = u16)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Sequence)]
pub enum PreferredHobby {
	#[default]
	None = 0x0,
	// these unknown values are probably created by some faulty tool
	// but they could also be an earlier encoding from earlier SP/EPs
	// an EP9 installation will reset these unknown values to random valid ones
	Unknown1 = 0x1,
	Unknown2 = 0x2,
	Unknown3 = 0x3,
	Unknown4 = 0x4,
	Unknown5 = 0x5,
	Unknown6 = 0x6,
	Unknown7 = 0x7,
	Unknown8 = 0x8,
	Unknown9 = 0x9,
	UnknownA = 0xa,
	UnknownCB = 0xcb,
	Cooking = 0xcc,
	Arts = 0xcd,
	Film = 0xce,
	Sports = 0xcf,
	Games = 0xd0,
	Nature = 0xd1,
	Tinkering = 0xd2,
	Fitness = 0xd3,
	Science = 0xd4,
	Music = 0xd5,
}

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct BugCollectionFlags {
	unknown_0: bool,

	pub grey_widow_spider: bool,
	pub striped_spindler_spider: bool,
	pub huntsperson_spider: bool,
	pub teddybear_spider: bool,
	pub itsius_bitsius_spider: bool,
	pub single_fanged_betsy_spider: bool,
	pub hotdog_spider: bool,
	pub queen_charlotte_spider: bool,
	pub paratrooper_spider: bool,
	pub mock_spider: bool,

	pub socialus_butterfly: bool,
	pub blue_featherwing_butterfly: bool,
	pub pygmalion_butterfly: bool,
	pub empress_butterfly: bool,
	pub jelly_butterfly: bool,
	pub peanut_butterfly: bool,
	pub margarina_butterfly: bool,
	pub copper_pot_butterfly: bool,
	pub vampire_butterfly: bool,
	pub madame_butterfly: bool,

	pub prancer_beetle: bool,
	pub jack_beetle: bool,
	pub mock_ladybug_beetle: bool,
	pub polka_beetle: bool,
	pub green_bottle_beetle: bool,
	pub dapper_pinstripe_beetle: bool,
	pub couch_potato_beetle: bool,
	pub ringo_beetle: bool,
	pub trihorn_greaves_beetle: bool,
	pub gentleman_beetle: bool,

	unknown_1: bool,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct FreeTimeData {
	pub hobbies_cooking: u16,
	pub hobbies_arts: u16,
	pub hobbies_film: u16,
	pub hobbies_sports: u16,
	pub hobbies_games: u16,
	pub hobbies_nature: u16,
	pub hobbies_tinkering: u16,
	pub hobbies_fitness: u16,
	pub hobbies_science: u16,
	pub hobbies_music: u16,
	pub hobbies_reserved: u16,
	pub preferred_hobby: PreferredHobby,
	pub lifetime_aspiration: u16,
	pub lifetime_aspiration_points: u16,
	pub lifetime_aspiration_points_spent: u16,
	pub decay_hunger_modifier: u16,
	pub decay_comfort_modifier: u16,
	pub decay_bladder_modifier: u16,
	pub decay_energy_modifier: u16,
	pub decay_hygiene_modifier: u16,
	pub decay_fun_modifier: u16,
	pub decay_social_modifier: u16,
	pub bugs_collection: BugCollectionFlags,
}
