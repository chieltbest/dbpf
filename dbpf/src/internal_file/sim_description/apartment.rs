use binrw::binrw;

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApartmentLifePreReleaseData {
	pub reputation: u16,
	pub probability_to_appear: u16,
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApartmentLifeData {
	pub title_post_name: u16,
}
