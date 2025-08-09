use binrw::binrw;

use crate::common::FileName;

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BehaviourConstants {
	pub file_name: FileName,

	#[br(temp)]
	#[bw(calc = constants.len() as u8)]
	count: u8,
	pub flag: u8,
	#[br(count = count)]
	pub constants: Vec<u16>,
}
