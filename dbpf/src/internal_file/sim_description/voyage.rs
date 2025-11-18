use binrw::binrw;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B12;

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageTraitFlags {
	pub robots: bool,
	pub plants: bool,
	pub lycanthropy: bool,
	pub witchiness: bool,
	unused: B12,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BonVoyageData {
	pub vacation_days_left: u16,
	pub turn_ons: BonVoyageTraitFlags,
	pub turn_offs: BonVoyageTraitFlags,
	pub traits: BonVoyageTraitFlags,
}
