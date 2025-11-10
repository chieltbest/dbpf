#![allow(unused_parens, clippy::identity_op)]

use crate::common::{Guid, SizedVec};
use binrw::binrw;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectID {
	id: u16,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Version {
	#[default]
	BaseGame = 0x20,
	University = 0x22,
	Nightlife = 0x29,
	Business = 0x2a,
	Pets = 0x2c,
	Castaway = 0x2d,
	BonVoyage = 0x2e,
	BonVoyageB = 0x2f,
	FreeTime = 0x33,
	ApartmentLife = 0x36,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UniProgressionFlags {
	year_1: bool,
	year_2: bool,
	year_3: bool,
	year_4: bool,
	semester: bool,
	try_period: bool,
	got_diploma: bool,
	in_course_or_exam: bool,
	unknown_0: bool,
	unknown_1: bool,
	unknown_2: bool,
	unknown_3: bool,
	abandon: bool,
	fired: bool,
	unknown_4: bool,
	unknown_5: bool,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UniData {
	uni_college_major_guid: Guid,
	uni_semester_remaining_time: u16,
	uni_progression_flags: UniProgressionFlags,
	uni_semester: u16,
	uni_on_campus: u16,
	uni_unknown: u32,
	uni_influence: u16,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NightlifeTraits {
	cologne: bool,
	stink: bool,
	fatness: bool,
	fitness: bool,
	formal_wear: bool,
	swim_wear: bool,
	underwear: bool,
	vampirism: bool,
	facial_hair: bool,
	glasses: bool,
	makeup: bool,
	full_face_makeup: bool,
	hats: bool,
	jewelry: bool,
	unused_0: bool,
	unused_1: bool,
	blonde_hair: bool,
	red_hair: bool,
	brown_hair: bool,
	black_hair: bool,
	custom_hair: bool,
	grey_hair: bool,
	hard_worker: bool,
	unemployed: bool,
	logical: bool,
	charismatic: bool,
	good_cook: bool,
	mechanical: bool,
	creative: bool,
	athletic: bool,
	good_cleaner: bool,
	zombiism: bool,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum Species {
	#[default]
	Human = 0,
	LargeDog = 1,
	SmallDog = 2,
	Cat = 3,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NightlifeData {
	route: u16,
	traits: NightlifeTraits,
	turn_ons: NightlifeTraits,
	turn_offs: NightlifeTraits,
	species: Species,
	countdown: u16,
	perfume_timer: u16,
	date_timer: u16,
	date_score: u16,
	date_unlock_counter: u16,
	love_potion_timer: u16,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BusinessData {
	lot_id: u16,
	salary: u16,
	flags: u16, // TODO bitfield
	assignment: u16,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageTraits {
	robots: bool,
	plants: bool,
	lycanthropy: bool,
	witchiness: bool,
	unused: B12,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageData {
	vacation_days_left: u16,
	turn_ons: BonVoyageTraits,
	turn_offs: BonVoyageTraits,
	traits: BonVoyageTraits,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum PreferredHobby {
	#[default]
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

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FreeTimeData {
	hobbies_cuisine: u16,
	hobbies_arts: u16,
	hobbies_film: u16,
	hobbies_sports: u16,
	hobbies_games: u16,
	hobbies_nature: u16,
	hobbies_tinkering: u16,
	hobbies_fitness: u16,
	hobbies_science: u16,
	hobbies_music: u16,
	unknown: u16,
	preferred_hobby: PreferredHobby,
	lifetime_aspiration: u16,
	lifetime_aspiration_points: u16,
	lifetime_aspiration_points_spent: u16,
	decay_hunger_modifier: u16,
	decay_comfort_modifier: u16,
	decay_bladder_modifier: u16,
	decay_energy_modifier: u16,
	decay_hygiene_modifier: u16,
	decay_fun_modifier: u16,
	decay_social_modifier: u16,
	bugs_collection: u32, // TODO bitfield
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApartmentLifeData {
	reputation: u16,
	probability_to_appear: u16,
	title_post_name: u16,
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SimDescription {
	unknown_0: u32,
	version: Version,
	#[br(temp, assert(version_repeat == version))]
	#[bw(calc(version.clone()))]
	version_repeat: Version,

	sitting: u16,
	money_over_head: u16,

	personality_nice: u16,
	personality_active: u16,
	/// also: personality - generosity
	uni_effort: u16,
	personality_playful: u16,
	personality_outgoing: u16,
	personality_neat: u16,
	current_outfit: u16,
	skill_cleaning: u16,
	skill_cooking: u16,
	skill_charisma: u16,
	skill_mechanical: u16,
	hot_date_exercise: u16,
	hot_date_food: u16,
	skill_creativity: u16,
	hot_date_parties: u16,
	skill_body: u16,
	skill_logic: u16,
	group_talk_state: u16,
	hot_date_style: u16,
	interaction_current_index: u16,
	preference_gender_male: u16,
	preference_gender_female: u16,
	job_data: u16,
	interaction_data_field_1: u16,
	interaction_sub_queue_count: u16,
	tick_counter: u16,
	interaction_data_field_2: u16,
	motives_static: u16,
	censorship_flags: u16, // TODO bitfield
	neighbor_id: u16,
	person_type: u16,
	priority: u16,
	greet_status: u16,
	visitor_schedule: u16,
	autonomy_level: u16,
	route_slot: u16,
	route_multi_slot_index: u16,
	route_status: u16,
	route_goal: u16,
	look_at_object_id: ObjectID,
	look_at_slot_id: u16,
	look_at_state: u16,
	look_at_time_remaining: u16,
	interaction_next_queued_index: u16,
	aspiration: u16,
	original_personality_neat: u16,
	original_personality_nice: u16,
	original_personality_active: u16,
	original_personality_outgoing: u16,
	original_personality_playful: u16,
	ui_icon_flags: u16, // TODO bitfield
	interaction_findbestaction_object_id: ObjectID,
	memory_score: u16,
	route_start_slot: u16,
	school_grade: u16,
	job_promotion_level: u16,
	age: u16,
	social_menu_object_id: ObjectID,
	skin_color: u16,
	family_number: u16,
	route_result: u16,
	job_performance: u16,
	swimming: u16,
	gender: u16,
	private: u16,
	lingering_house_number: u16,
	route_ghost_flags: u16, // TODO bitfield
	job_pto: u16,
	zodiac: u16,
	non_interruptable: u16,
	interaction_next_queued_continuation: u16,
	footprint_extension: u16,
	render_display_flags: u16, // TODO bitfield
	interaction_sub_queue_master_object_id: ObjectID,
	interaction_sub_queue_master_interaction_index: u16,
	interaction_sub_queue_next_interaction_index: u16,
	interaction_sub_queue_next_interaction_object_id: ObjectID,
	interaction_sub_queue_current_interaction_object_id: ObjectID,
	body_flags: u16, // TODO bitfield
	fatness: u16,
	/// also: life score toddler
	uni_grade: u16,
	life_score_child: u16,
	life_score_teen: u16,
	life_score_adult: u16,
	life_score_elder: u16,
	voice_type: u16,
	job_object_guid: Guid,
	age_days_left: u16,
	age_previous_days: u16,
	/// per day
	decay_hunger: u16,
	/// per day
	decay_comfort: u16,
	/// per day
	decay_bladder: u16,
	/// per day
	decay_energy: u16,
	/// per day
	decay_hygiene: u16,
	/// per day
	decay_social_family: u16,
	/// per day
	decay_social: u16,
	/// per day
	decay_unknown: u16,
	/// per day
	decay_fun: u16,
	interaction_current_running_index: u16,
	interaction_current_running_object_id: ObjectID,
	genetics_data_1: u16,
	genetics_data_2: u16,
	genetics_data_3: u16,
	school_object_guid: Guid,
	interaction_current_guid: u16,
	interaction_linked_deleted: u16,
	skill_romance: u16,
	loco_weight_0: u16,
	loco_weight_1: u16,
	loco_personality_index: u16,
	loco_personality_weight: u16,
	loco_mood_index: u16,
	loco_mood_weight: u16,
	loco_motives: u16, // TODO bitfield?
	outfit_source_guid: Guid,
	environment_score_override: u16,
	fitness_preference: u16,
	pension: u16,
	interest_politics: u16,
	interest_money: u16,
	interest_environment: u16,
	interest_crime: u16,
	interest_entertainment: u16,
	interest_culture: u16,
	interest_food: u16,
	interest_health: u16,
	interest_fashion: u16,
	interest_sports: u16,
	interest_paranormal: u16,
	interest_travel: u16,
	interest_work: u16,
	interest_weather: u16,
	interest_animals: u16,
	interest_school: u16,
	interest_toys: u16,
	interest_scifi: u16,
	interest_unused_0: u16,
	interest_unused_1: u16,
	interest_unused_2: u16,
	interest_unused_3: u16,
	interest_unused_4: u16,
	interest_unused_5: u16,
	interest_unused_6: u16,
	interest_unused_7: u16,
	interest_unused_8: u16,
	interest_unused_9: u16,
	interest_unused_10: u16,
	interest_unused_11: u16,
	unselectable: u16,
	npc_type: u16,
	age_duration: u16,
	interaction_sub_queue_object_id: ObjectID,
	selection_flags: u16,
	person_flags: u16,
	aspiration_score: u16,
	/// divide by 10
	aspiration_reward_points_spent: u16,
	/// divide by 10
	aspiration_score_raw: u16,
	mood_booster: u16,
	interaction_current_joinable: u16,
	unlinked: u16,
	interaction_autonomous: u16,
	job_retired_guid: Guid,
	job_retired_level: u16,

	#[brw(if(version.clone() >= Version::University))]
	uni_data: UniData,

	#[brw(if(version.clone() >= Version::Nightlife))]
	nightlife_data: NightlifeData,

	#[brw(if(version.clone() >= Version::Business))]
	open_for_business_data: BusinessData,

	#[brw(if(version.clone() >= Version::Pets))]
	pet_traits: u16,

	#[brw(if(version.clone() >= Version::BonVoyage))]
	bon_voyage_data: BonVoyageData,

	#[brw(if(version.clone() >= Version::Castaway))]
	subspecies: u16,

	#[brw(if(version.clone() >= Version::FreeTime))]
	free_time_data: FreeTimeData,

	#[brw(if(version.clone() >= Version::ApartmentLife))]
	apartment_life_data: ApartmentLifeData,

	instance: u16,
	guid: Guid,

	unknown_1: u32,

	relations: SizedVec<u32, u16>,

	unknown_2: [u8; 9],
}
