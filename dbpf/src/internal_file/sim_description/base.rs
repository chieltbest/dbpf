use crate::internal_file::sim_description::SimID;
use binrw::binrw;
use enum_iterator::Sequence;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::{B11, B12, B14, B4, B7};

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Sequence)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Sequence)]
pub enum Gender {
	Male = 0,
	#[default]
	Female = 1,
}

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Sequence)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Sequence)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Sequence)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectionFlags {
	pub selectable: bool,
	pub not_selectable: bool,
	pub hide_relationships: bool,
	pub holiday_mate: bool,
	unused: B12,
}

#[bitfield]
#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct PersonFlags1 {
	pub is_owned: bool,
	pub stay_naked: bool,
	unused: B14,
}

#[binrw]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SimRelation {
	pub relation: SimID,
	pub unknown: u16,
}
