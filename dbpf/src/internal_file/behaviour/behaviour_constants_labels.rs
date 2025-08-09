use binrw::{args, binrw};

use crate::common::{BigString, FileName, NullString};

#[binrw]
#[brw(magic = b"NCRT")]
struct HeaderMagic;

#[binrw]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Header {
	Normal(
		#[br(temp)]
		#[bw(calc = HeaderMagic)]
		HeaderMagic,
		u32, // TODO enum
		u32,
	),
	ExtraNulls(
		u32,
		u32,
		#[br(temp)]
		#[bw(calc = HeaderMagic)]
		HeaderMagic,
	),
}

impl Default for Header {
	fn default() -> Self {
		Self::Normal(0, 0)
	}
}

#[binrw]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LabelV1 {
	pub used: u32,
	pub id: u32,
	pub name: BigString,
	pub default: u16,
	pub min: u16,
	pub max: u16,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum LabelV2Tag {
	#[default]
	Some = 0xa3,
}

#[binrw]
#[brw(import { header: Header })]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LabelV2 {
	pub used: u32,
	pub id: u32,
	pub name: NullString,
	#[br(try)]
	pub unknown: Option<LabelV2Tag>,
	pub description: NullString,
	#[br(try)]
	pub unknown2: Option<LabelV2Tag>,
	#[brw(if(matches!(header, Header::ExtraNulls(_, 1))))]
	pub unknown3: [u8; 5],
}

#[binrw]
#[brw(import { header: Header })]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Label {
	#[br(pre_assert(matches!(header, Header::Normal(_, _))))]
	V1(LabelV1),
	#[br(pre_assert(matches!(header, Header::ExtraNulls(_, _))))]
	V2(#[brw(args { header: header })] LabelV2),
}

impl Default for Label {
	fn default() -> Self {
		Self::V1(LabelV1::default())
	}
}

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BehaviourConstantsLabels {
	pub file_name: FileName,

	pub header: Header,

	#[br(temp)]
	#[bw(calc = labels.len() as u32)]
	count: u32,
	#[br(count = count, args { inner: args! { header: header } })]
	#[bw(args { header: *header })]
	pub labels: Vec<Label>,
}
