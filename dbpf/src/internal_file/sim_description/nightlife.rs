// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;
use modular_bitfield::bitfield;

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct NightlifeTraitFlags {
	pub cologne: bool,
	pub stink: bool,
	pub fatness: bool,
	pub fitness: bool,
	pub formal_wear: bool,
	pub swim_wear: bool,
	pub underwear: bool,
	pub vampirism: bool,
	pub facial_hair: bool,
	pub glasses: bool,
	pub makeup: bool,
	pub full_face_makeup: bool,
	pub hats: bool,
	pub jewelry: bool,
	unused_0: bool,
	unused_1: bool,
	pub blonde_hair: bool,
	pub red_hair: bool,
	pub brown_hair: bool,
	pub black_hair: bool,
	pub custom_hair: bool,
	pub grey_hair: bool,
	pub hard_worker: bool,
	pub unemployed: bool,
	pub logical: bool,
	pub charismatic: bool,
	pub good_cook: bool,
	pub mechanical: bool,
	pub creative: bool,
	pub athletic: bool,
	pub good_cleaner: bool,
	pub zombiism: bool,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Species {
	#[default]
	Human = 0,
	LargeDog = 1,
	SmallDog = 2,
	Cat = 3,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct NightlifeData {
	pub route: u16,
	pub traits: NightlifeTraitFlags,
	pub turn_ons: NightlifeTraitFlags,
	pub turn_offs: NightlifeTraitFlags,
	pub species: Species,
	pub countdown: u16,
	pub perfume_timer: u16,
	pub date_timer: u16,
	pub date_score: u16,
	pub date_unlock_counter: u16,
	pub love_potion_timer: u16,
	pub aspiration_score_lock: u16,
	pub date_neighbor_id: u16,
}
