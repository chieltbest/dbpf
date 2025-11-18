use binrw::binrw;
use enum_iterator::Sequence;

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApartmentLifePreReleaseData {
	/// -100 - 100
	pub reputation: i16,
	pub probability_to_appear: u16,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Sequence)]
pub enum TitlePostName {
	#[default]
	None = 0x0,
	AtrociouslyEvilWitch = 0x1,
	AtrociouslyEvilWarlock = 0x2,
	EvilWitch = 0x3,
	EvilWarlock = 0x4,
	MeanWitch = 0x5,
	MeanWarlock = 0x6,
	Witch = 0x7,
	Warlock = 0x8,
	NiceWitch = 0x9,
	NiceWarlock = 0xA,
	GoodWitch = 0xB,
	GoodWarlock = 0xC,
	InfalliblyGoodWitch = 0xD,
	InfalliblyGoodWarlock = 0xE,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApartmentLifeData {
	pub title_post_name: TitlePostName,
}
