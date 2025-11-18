use binrw::binrw;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::B6;

#[bitfield]
#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PetTraitFlags {
	pub gifted: bool,
	pub doofus: bool,
	pub hyper: bool,
	pub lazy: bool,
	pub independent: bool,
	pub friendly: bool,
	pub aggressive: bool,
	pub cowardly: bool,
	pub pigpen: bool,
	pub finicky: bool,
	unused: B6,
}
