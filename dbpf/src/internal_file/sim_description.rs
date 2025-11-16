#![allow(unused_parens, clippy::identity_op)]

use crate::common::{Guid, SizedVec};
use binrw::binrw;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;

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
	/// SP8?
	V35 = 0x35,
	/// EP8
	ApartmentLife = 0x36,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AspirationFlags {
	romance: bool,
	family: bool,
	fortune: bool,
	power: bool, // TODO real?
	reputation: bool,
	knowledge: bool,
	grow_up: bool,
	pleasure: bool,
	cheese: bool,
	unused: B7,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum Grade {
	#[default]
	Unknown = 0x0,
	F = 0x1,
	DMinus = 0x2,
	D = 0x3,
	DPlus = 0x4,
	CMinus = 0x5,
	C = 0x6,
	CPlus = 0x7,
	BMinus = 0x8,
	B = 0x9,
	BPlus = 0xA,
	AMinus = 0xB,
	A = 0xC,
	APlus = 0xD,
}

#[binrw]
#[brw(repr = u32)]
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
#[brw(repr = u32)]
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
	is_ghost: bool,
	can_pass_through_objects: bool,
	can_pass_through_walls: bool,
	can_pass_through_people: bool,
	ignore_traversal_costs: bool,
	can_fly_over_low_objects: bool,
	force_route_recalc: bool,
	can_swim_in_ocean: bool,
	unused: u8,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum ZodiacSign {
	#[default]
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
	fat: bool,
	pregnant_3rd_trimester: bool,
	pregnant_2nd_trimester: bool,
	pregnant_1st_trimester: bool,
	fit: bool,
	hospital: bool,
	birth_control: bool,
	unused0: bool,
	unused1: u8,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionFlags {
	selectable: bool,
	not_selectable: bool,
	hide_relationships: bool,
	holiday_mate: bool,
	unused: B12,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PersonFlags0 {
	zombie: bool,
	perma_platinum: bool,
	is_vampire: bool,
	vampire_smoke: bool,
	want_history: bool,
	lycanthropy_carrier: bool,
	lycanthropy_active: bool,
	is_pet_runaway: bool,
	is_plantsim: bool,
	is_bigfoot: bool,
	is_witch: bool,
	is_roommate: bool,
	unused: B4,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PersonFlags1 {
	is_owned: bool,
	stay_naked: bool,
	unused: B14,
}

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
	pub unknown_0: bool,
	pub unknown_1: bool,
	pub unknown_2: bool,
	pub unknown_3: bool,
	pub abandon: bool,
	pub fired: bool,
	pub unknown_4: bool,
	pub unknown_5: bool,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UniData {
	pub uni_college_major_guid: Guid,
	pub uni_semester_remaining_time: u16,
	pub uni_progression_flags: UniProgressionFlags,
	pub uni_semester: u16,
	pub uni_on_campus: u16,
	pub uni_unknown: u32,
	pub uni_influence: u16,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NightlifeTraits {
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
	pub unused_0: bool,
	pub unused_1: bool,
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
	pub route: u16,
	pub traits: NightlifeTraits,
	pub turn_ons: NightlifeTraits,
	pub turn_offs: NightlifeTraits,
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

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum JobAssignment {
	#[default]
	Nothing = 0x0,
	Chef = 0x1,
	Host = 0x2,
	Server = 0x3,
	Cashier = 0x4,
	Bartender = 0x5,
	Barista = 0x6,
	DJ = 0x7,
	SellLemonade = 0x8,
	Stylist = 0x9,
	Tidy = 0xA,
	Restock = 0xB,
	Sales = 0xC,
	MakeToys = 0xD,
	ArrangeFlowers = 0xE,
	BuildRobots = 0xF,
	MakeFood = 0x10,
	Masseuse = 0x11,
	MakePottery = 0x12,
	Sewing = 0x13,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BusinessData {
	pub lot_id: u16,
	pub salary: u16,
	pub flags: u16, // TODO bitfield
	pub assignment: JobAssignment,
}

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageTraits {
	pub robots: bool,
	pub plants: bool,
	pub lycanthropy: bool,
	pub witchiness: bool,
	pub unused: B12,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageData {
	pub vacation_days_left: u16,
	pub turn_ons: BonVoyageTraits,
	pub turn_offs: BonVoyageTraits,
	pub traits: BonVoyageTraits,
}

#[binrw]
#[brw(repr = u16)]
#[brw(import(version: Version))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
	pub unknown: u16,
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
	pub bugs_collection: u32, // TODO bitfield
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct V35Data {
	pub reputation: u16,
	pub probability_to_appear: u16,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApartmentLifeData {
	pub title_post_name: u16,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SimRelation {
	pub relation: SimID,
	pub unknown: u16,
}

// TODO CultFlags
// TODO PetTraits

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
	pub hot_date_style: u16,
	pub interaction_current_index: u16,
	pub preference_gender_male: u16,
	pub preference_gender_female: u16,
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
	pub school_grade: Grade,
	pub job_promotion_level: u16,
	pub age: LifeSection,
	pub social_menu_object_id: ObjectID,
	pub skin_color: u16,
	pub family_number: u16,
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
	pub genetics_data_1: u16,
	pub genetics_data_2: u16,
	pub genetics_data_3: u16,
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
	pub interest_unused_2: u16,
	pub interest_unused_3: u16,
	pub interest_unused_4: u16,
	pub interest_unused_5: u16,
	pub interest_unused_6: u16,
	pub interest_unused_7: u16,
	pub interest_unused_8: u16,
	pub interest_unused_9: u16,
	pub interest_unused_10: u16,
	pub interest_unused_11: u16,
	pub unselectable: u16,
	pub npc_type: u16, // TODO enum
	pub age_duration: u16,
	pub interaction_sub_queue_object_id: ObjectID,
	pub selection_flags: SelectionFlags,
	pub person_flags_1: PersonFlags1,
	// also: bodyshape???
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
	pub pet_traits: u16, // TODO bitfield

	#[brw(if(version.clone() >= Version::BonVoyage))]
	pub bon_voyage_data: BonVoyageData,

	#[brw(if(version.clone() == Version::Castaway))]
	pub subspecies: u16,

	#[brw(if(version.clone() >= Version::FreeTime))]
	pub free_time_data: FreeTimeData,

	#[brw(if(version.clone() >= Version::V35))]
	pub v35_data: V35Data,

	#[brw(if(version.clone() >= Version::ApartmentLife))]
	pub apartment_life_data: ApartmentLifeData,

	pub instance: SimID,
	pub guid: Guid,

	#[br(assert(unknown_1 == 3))]
	pub unknown_1: u32,

	pub relations: SizedVec<u32, SimRelation>,

	#[brw(if(version.clone() >= Version::BonVoyage))]
	pub collectibles: u64, // TODO flags

	pub unknown_3: u8,
}
