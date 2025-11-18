// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_parens, clippy::identity_op)]

use crate::common::{Guid, SizedVec};
use crate::internal_file::sim_description::apartment::{
	ApartmentLifeData, ApartmentLifePreReleaseData,
};
use crate::internal_file::sim_description::business::BusinessData;
use crate::internal_file::sim_description::freetime::FreeTimeData;
use crate::internal_file::sim_description::nightlife::NightlifeData;
use crate::internal_file::sim_description::pets::PetTraitFlags;
use crate::internal_file::sim_description::university::UniData;
use crate::internal_file::sim_description::voyage::{BonVoyageData, BonVoyageMementosFlags};
use binrw::binrw;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

mod apartment;
mod business;
mod freetime;
mod nightlife;
mod pets;
mod university;
mod voyage;

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectID {
	pub id: u16,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SimID {
	pub id: u16,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Version {
	/// repeat only
	V18 = 0x18,
	/// repeat only
	V19 = 0x19,
	/// repeat only
	V1a = 0x1a,
	/// repeat only
	V1c = 0x1c,
	/// repeat only
	V1e = 0x1e,
	/// repeat only
	V1f = 0x1f,
	#[default]
	/// Base Game
	BaseGame = 0x20,
	/// EP1
	University = 0x22,
	/// repeat only
	V27 = 0x27,
	/// EP2
	Nightlife = 0x29,
	/// EP3
	Business = 0x2a,
	/// EP4
	Pets = 0x2c,
	/// castaway/seasons?
	Castaway = 0x2d,
	/// EP6
	BonVoyage = 0x2e,
	/// store edition?
	BonVoyageB = 0x2f,
	/// EP7
	FreeTime = 0x33,
	/// pre-release of EP8? only used for four pets in the magic subhood
	ApartmentLifePreRelease = 0x35,
	/// EP8
	ApartmentLife = 0x36,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AspirationFlags {
	pub romance: bool,
	pub family: bool,
	pub fortune: bool,
	pub power: bool, // TODO real?
	pub reputation: bool,
	pub knowledge: bool,
	pub grow_up: bool,
	pub pleasure: bool,
	pub grilled_cheese: bool,
	unused: B7,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum LifeSection {
	#[default]
	Unknown = 0x0,
	Baby = 0x1,
	Toddler = 0x2,
	Child = 0x3,
	Teen = 0x10,
	Adult = 0x13,
	Elder = 0x33,
	YoungAdult = 0x40,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Gender {
	Male = 0,
	#[default]
	Female = 1,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GhostFlags {
	pub is_ghost: bool,
	pub can_pass_through_objects: bool,
	pub can_pass_through_walls: bool,
	pub can_pass_through_people: bool,
	pub ignore_traversal_costs: bool,
	pub can_fly_over_low_objects: bool,
	pub force_route_recalc: bool,
	pub can_swim_in_ocean: bool,
	unused: u8,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum ZodiacSign {
	#[default]
	Unknown = 0,
	Aries = 1,
	Taurus = 2,
	Gemini = 3,
	Cancer = 4,
	Leo = 5,
	Virgo = 6,
	Libra = 7,
	Scorpio = 8,
	Sagittarius = 9,
	Capricorn = 10,
	Aquarius = 11,
	Pices = 12,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BodyFlags {
	pub fat: bool,
	pub pregnant_3rd_trimester: bool,
	pub pregnant_2nd_trimester: bool,
	pub pregnant_1st_trimester: bool,
	pub fit: bool,
	pub hospital: bool,
	pub birth_control: bool,
	unused0: bool,
	unused1: u8,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum BodyShape {
	#[default]
	Default = 0x0,
	Tiny = 0x13,
	Elder = 0x15,
	Maxis = 0x1e,
	Holiday = 0x1f,
	Goth = 0x20,
	SteamPunk = 0x21,
	Medieval = 0x22,
	StoneAge = 0x23,
	Pirates = 0x24,
	Grungy = 0x26,
	Castaway = 0x27,
	SuperHeros = 0x29,
	Futuristic = 0x2a,
	Various = 0x2c,
	Werewolves = 0x2d,
	Satyrs = 0x2f,
	Centaurs = 0x30,
	Mermaid = 0x31,
	HugeBBBeast = 0x33,
	Fannystein = 0x35,
	Quarians = 0x36,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CultFlags {
	pub allow_family: bool,
	pub no_alcohol: bool,
	pub no_auto_woohoo: bool,
	pub marked_sim: bool,
	pub not_used_f: bool, // TODO ?
	unused: B11,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum NpcType {
	#[default]
	Normal = 0x0,
	BartenderBars = 0x1,
	BartenderPhone = 0x2,
	Boss = 0x3,
	Burglar = 0x4,
	Driver = 0x5,
	Streaker = 0x6,
	Coach = 0x7,
	LunchLady = 0x8,
	Cop = 0x9,
	Delivery = 0xA,
	Exterminator = 0xB,
	FireFighter = 0xC,
	Gardener = 0xD,
	Barista = 0xE,
	Grim = 0xF,
	Handy = 0x10,
	Headmistress = 0x11,
	Matchmaker = 0x12,
	Maid = 0x13,
	MailCarrier = 0x14,
	Nanny = 0x15,
	Paper = 0x16,
	Pizza = 0x17,
	Professor = 0x18,
	EvilMascot = 0x19,
	Repo = 0x1A,
	CheerLeader = 0x1B,
	Mascot = 0x1C,
	SocialBunny = 0x1D,
	SocialWorker = 0x1E,
	Register = 0x1F,
	Therapist = 0x20,
	Chinese = 0x21,
	Podium = 0x22,
	Waitress = 0x23,
	Chef = 0x24,
	DJ = 0x25,
	Crumplebottom = 0x26,
	Vampyre = 0x27,
	Servo = 0x28,
	Reporter = 0x29,
	Salon = 0x2A,
	Wolf = 0x2B,
	WolfLOTP = 0x2C,
	Skunk = 0x2D,
	AnimalControl = 0x2E,
	Obedience = 0x2F,
	Masseuse = 0x30,
	Bellhop = 0x31,
	Villain = 0x32,
	TourGuide = 0x33,
	Hermit = 0x34,
	Ninja = 0x35,
	BigFoot = 0x36,
	Housekeeper = 0x37,
	FoodStandChef = 0x38,
	FireDancer = 0x39,
	WitchDoctor = 0x3A,
	GhostCaptain = 0x3B,
	FoodJudge = 0x3C,
	Genie = 0x3D,
	ExDj = 0x3E,
	ExGypsy = 0x3F,
	Witch1 = 0x40,
	Breakdancer = 0x41,
	SpectralCat = 0x42,
	Statue = 0x43,
	Landlord = 0x44,
	Butler = 0x45,
	HotdogChef = 0x46,
	Assistant = 0x47,
	ExWitch2 = 0x48,
	TinySim = 0x4F,
	Pandora = 0xAC,
	DMASim = 0xDA,
	Icontrol = 0xE9,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionFlags {
	pub selectable: bool,
	pub not_selectable: bool,
	pub hide_relationships: bool,
	pub holiday_mate: bool,
	unused: B12,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PersonFlags0 {
	pub zombie: bool,
	pub perma_platinum: bool,
	pub is_vampire: bool,
	pub vampire_smoke: bool,
	pub want_history: bool,
	pub lycanthropy_carrier: bool,
	pub lycanthropy_active: bool,
	pub is_pet_runaway: bool,
	pub is_plantsim: bool,
	pub is_bigfoot: bool,
	pub is_witch: bool,
	pub is_roommate: bool,
	unused: B4,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PersonFlags1 {
	pub is_owned: bool,
	pub stay_naked: bool,
	unused: B14,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SimRelation {
	pub relation: SimID,
	pub unknown: u16,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SimDescription {
	pub unknown_0: u32,
	pub version: Version,
	pub version_repeat: Version,

	pub sitting: u16,
	pub money_over_head: u16,
	pub personality_nice: u16,
	pub personality_active: u16,
	/// also: personality - generosity
	pub uni_effort: u16,
	pub personality_playful: u16,
	pub personality_outgoing: u16,
	pub personality_neat: u16,
	pub current_outfit: u16,
	pub skill_cleaning: u16,
	pub skill_cooking: u16,
	pub skill_charisma: u16,
	pub skill_mechanical: u16,
	/// also: hot date exercise
	/// also: LazyDutchess' height mod height
	pub skill_music: u16,
	/// also: hot date food
	pub partner_id: SimID,
	pub skill_creativity: u16,
	/// also: hot date parties
	pub skill_art: u16,
	pub skill_body: u16,
	pub skill_logic: u16,
	pub group_talk_state: u16, // TODO bitfield
	/// also: hot date style
	pub body_temperature: u16,
	pub interaction_current_index: u16,
	pub preference_gender_male: i16,
	pub preference_gender_female: i16,
	pub job_data: u16, // TODO bitfield
	pub interaction_data_field_1: u16,
	pub interaction_sub_queue_count: u16,
	pub tick_counter: u16,
	pub interaction_data_field_2: u16,
	pub motives_static: u16,
	pub censorship_flags: u16, // TODO bitfield
	pub neighbor_id: u16,      // TODO SimID?
	pub person_type: u16,
	pub priority: u16,
	pub greet_status: u16,
	pub visitor_schedule: u16,
	pub autonomy_level: u16,
	pub route_slot: u16,
	pub route_multi_slot_index: u16,
	pub route_status: u16,
	pub route_goal: u16,
	pub look_at_object_id: ObjectID,
	pub look_at_slot_id: u16,
	pub look_at_state: u16,
	pub look_at_time_remaining: u16,
	pub interaction_next_queued_index: u16,
	pub aspiration: AspirationFlags,
	pub genetic_personality_neat: u16,
	pub genetic_personality_nice: u16,
	pub genetic_personality_active: u16,
	pub genetic_personality_outgoing: u16,
	pub genetic_personality_playful: u16,
	pub ui_icon_flags: u16, // TODO bitfield
	pub interaction_findbestaction_object_id: ObjectID,
	pub memory_score: u16,
	pub route_start_slot: u16,
	/// seems to be 0-1000
	/// SimPE suggests this is an enum
	pub school_grade: u16,
	pub job_promotion_level: u16,
	pub age: LifeSection,
	pub social_menu_object_id: ObjectID,
	pub skin_color: u16,
	pub family_instance: u16,
	pub route_result: u16,
	pub job_performance: u16,
	pub swimming: u16,
	pub gender: Gender,
	pub private: u16,
	pub lingering_house_number: u16,
	pub route_ghost_flags: GhostFlags,
	pub job_pto: u16,
	pub zodiac: ZodiacSign,
	pub non_interruptable: u16,
	pub interaction_next_queued_continuation: u16,
	pub footprint_extension: u16,
	pub render_display_flags: u16, // TODO bitfield
	pub interaction_sub_queue_master_object_id: ObjectID,
	pub interaction_sub_queue_master_interaction_index: u16,
	pub interaction_sub_queue_next_interaction_index: u16,
	pub interaction_sub_queue_next_interaction_object_id: ObjectID,
	pub interaction_queue_next_interaction_object_id: ObjectID,
	pub interaction_queue_current_interaction_object_id: ObjectID,
	pub body_flags: BodyFlags,
	/// 0-1000
	pub fatness: u16,
	/// also: life score toddler
	pub uni_grade: u16,
	/// also: life score child
	pub person_flags_0: PersonFlags0,
	/// also: romance skill?
	pub life_score_teen: u16,
	pub life_score_adult: u16,
	/// also: voice type?
	pub life_score_elder: u16,
	pub voice_type: u16,
	pub job_object_guid: Guid,
	pub age_days_left: u16,
	pub age_previous_days: u16,
	/// per day
	pub decay_hunger: u16,
	/// per day
	pub decay_comfort: u16,
	/// per day
	pub decay_bladder: u16,
	/// per day
	pub decay_energy: u16,
	/// per day
	pub decay_hygiene: u16,
	/// per day
	pub decay_social_family: u16,
	/// per day
	pub decay_social: u16,
	/// per day
	pub decay_unknown: u16,
	/// per day
	pub decay_fun: u16,
	pub interaction_current_running_index: u16,
	pub interaction_current_running_object_id: ObjectID,
	/// also: genetics data 1
	pub motive_power: u16,
	/// also: genetics data 2
	pub work_outfit_index: u16,
	/// per day
	/// also: genetics data 3
	pub decay_scratch_chew: u16,
	pub school_object_guid: Guid,
	pub interaction_current_guid: u16,
	pub interaction_linked_deleted: u16,
	pub skill_romance: u16,
	pub loco_weight_0: u16,
	pub loco_weight_1: u16,
	pub loco_personality_index: u16,
	pub loco_personality_weight: u16,
	pub loco_mood_index: u16,
	pub loco_mood_weight: u16,
	pub loco_motives: u16, // TODO bitfield?
	pub outfit_source_guid: Guid,
	pub environment_score_override: u16,
	pub fitness_preference: u16,
	pub pension: u16,
	pub interest_politics: u16,
	pub interest_money: u16,
	pub interest_environment: u16,
	pub interest_crime: u16,
	pub interest_entertainment: u16,
	pub interest_culture: u16,
	pub interest_food: u16,
	pub interest_health: u16,
	pub interest_fashion: u16,
	pub interest_sports: u16,
	pub interest_paranormal: u16,
	pub interest_travel: u16,
	pub interest_work: u16,
	pub interest_weather: u16,
	pub interest_animals: u16,
	pub interest_school: u16,
	pub interest_toys: u16,
	pub interest_scifi: u16,
	pub interest_unused_0: u16,
	pub interest_unused_1: u16,
	/// also: interest_unused_2
	pub allocated_suburb: u16,
	/// also: interest_unused_3
	pub person_flags_1: PersonFlags1,
	/// also: interest_unused_4
	pub bodyshape: BodyShape,
	pub interest_unused_5: u16,
	/// also: interest_unused_6
	pub cult_flags: CultFlags,
	pub interest_unused_7: u16,
	pub interest_unused_8: u16,
	pub interest_unused_9: u16,
	/// also: interest_unused_10
	pub religion_id: u16,
	pub interest_unused_11: u16,
	pub unselectable: u16,
	pub npc_type: NpcType, // TODO enum
	pub age_duration: u16,
	pub interaction_sub_queue_object_id: ObjectID,
	pub selection_flags: SelectionFlags,
	pub person_flags_2: u16, // TODO bitfield
	pub aspiration_score: u16,
	/// divide by 10
	pub aspiration_reward_points_spent: u16,
	/// divide by 10
	pub aspiration_score_raw: u16,
	pub mood_booster: u16,
	pub interaction_current_joinable: u16,
	pub unlinked: u16,
	pub interaction_autonomous: u16,
	pub job_retired_guid: Guid,
	pub job_retired_level: u16,

	#[brw(if(version.clone() >= Version::University))]
	pub uni_data: UniData,

	#[brw(if(version.clone() >= Version::Nightlife))]
	pub nightlife_data: NightlifeData,

	#[brw(if(version.clone() >= Version::Business))]
	pub open_for_business_data: BusinessData,

	#[brw(if(version.clone() >= Version::Pets))]
	pub pet_traits: PetTraitFlags,

	#[brw(if(version.clone() >= Version::BonVoyage))]
	pub bon_voyage_data: BonVoyageData,

	#[brw(if(version.clone() == Version::Castaway))]
	pub subspecies: u16,

	#[brw(if(version.clone() >= Version::FreeTime))]
	pub free_time_data: FreeTimeData,

	#[brw(if(version.clone() >= Version::ApartmentLifePreRelease))]
	pub v35_data: ApartmentLifePreReleaseData,

	#[brw(if(version.clone() >= Version::ApartmentLife))]
	pub apartment_life_data: ApartmentLifeData,

	pub instance: SimID,
	pub guid: Guid,

	#[br(assert(unknown_1 == 3))]
	pub unknown_1: u32,

	pub relations: SizedVec<u32, SimRelation>,

	pub unknown_3: u8,

	#[brw(if(version.clone() >= Version::BonVoyage))]
	pub mementos: BonVoyageMementosFlags,
}
