// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_parens, clippy::identity_op)]

use crate::common::{Guid, SizedVec};
use crate::internal_file::sim_description::apartment::{
	ApartmentLifeData, ApartmentLifePreReleaseData,
};
use crate::internal_file::sim_description::base::{
	AspirationFlags, BodyFlags, BodyShape, CultFlags, Gender, GhostFlags, LifeSection, NpcType,
	PersonFlags0, PersonFlags1, SelectionFlags, SimRelation, ZodiacSign,
};
use crate::internal_file::sim_description::business::BusinessData;
use crate::internal_file::sim_description::freetime::FreeTimeData;
use crate::internal_file::sim_description::nightlife::NightlifeData;
use crate::internal_file::sim_description::pets::PetTraitFlags;
use crate::internal_file::sim_description::university::UniData;
use crate::internal_file::sim_description::voyage::{BonVoyageData, BonVoyageMementosFlags};
use binrw::binrw;
use enum_iterator::Sequence;

pub mod apartment;
pub mod base;
pub mod business;
pub mod freetime;
pub mod nightlife;
pub mod pets;
pub mod university;
pub mod voyage;

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectID {
	pub id: u16,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SimID {
	pub id: u16,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Sequence)]
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

impl Version {
	pub fn human_name(&self) -> &str {
		match self {
			Version::V18 => "V18",
			Version::V19 => "V19",
			Version::V1a => "V1a",
			Version::V1c => "V1c",
			Version::V1e => "V1e",
			Version::V1f => "V1f",
			Version::BaseGame => "Base Game",
			Version::University => "University",
			Version::V27 => "V27",
			Version::Nightlife => "Nightlife",
			Version::Business => "Business",
			Version::Pets => "Pets",
			Version::Castaway => "Castaway",
			Version::BonVoyage => "Bon Voyage",
			Version::BonVoyageB => "Bon Voyage (Version 2)",
			Version::FreeTime => "Free Time",
			Version::ApartmentLifePreRelease => "Apartment Life (pre-release)",
			Version::ApartmentLife => "Apartment Life",
		}
	}
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
	pub body_temperature: i16,
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
	pub interaction_sub_queue_master_interaction_object_id: ObjectID,
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
	pub decay_hunger: i16,
	/// per day
	pub decay_comfort: i16,
	/// per day
	pub decay_bladder: i16,
	/// per day
	pub decay_energy: i16,
	/// per day
	pub decay_hygiene: i16,
	/// per day
	pub decay_social_family: i16,
	/// per day
	pub decay_social: i16,
	/// per day
	pub decay_shopping: i16,
	/// per day
	pub decay_fun: i16,
	pub interaction_current_running_index: u16,
	pub interaction_current_running_object_id: ObjectID,
	/// also: genetics data 1
	pub motive_power: u16,
	/// also: genetics data 2
	pub work_outfit_index: u16,
	/// per day
	/// also: genetics data 3
	pub decay_scratch_chew: i16,
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

	#[brw(if(version.ge(&Version::University)))]
	pub uni_data: UniData,

	#[brw(if(version.ge(&Version::Nightlife)))]
	pub nightlife_data: NightlifeData,

	#[brw(if(version.ge(&Version::Business)))]
	pub business_data: BusinessData,

	#[brw(if(version.ge(&Version::Pets)))]
	pub pet_traits: PetTraitFlags,

	#[brw(if(version.ge(&Version::BonVoyage)))]
	pub bon_voyage_data: BonVoyageData,

	#[brw(if(version.eq(&Version::Castaway)))]
	pub subspecies: u16,

	#[brw(if(version.ge(&Version::FreeTime)))]
	pub free_time_data: FreeTimeData,

	#[brw(if(version.ge(&Version::ApartmentLifePreRelease)))]
	pub apartment_life_pre_release_data: ApartmentLifePreReleaseData,

	#[brw(if(version.ge(&Version::ApartmentLife)))]
	pub apartment_life_data: ApartmentLifeData,

	pub instance: SimID,
	pub guid: Guid,

	#[br(assert(unknown_1 == 3))]
	pub unknown_1: u32,

	pub relations: SizedVec<u32, SimRelation>,

	pub unknown_3: u8,

	#[brw(if(version.ge(&Version::BonVoyage)))]
	pub mementos: BonVoyageMementosFlags,
}
