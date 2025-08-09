use binrw::binrw;

use crate::common::{BigString, FileName};

#[binrw]
#[brw(magic = b"PRPT")]
struct Header;

#[binrw]
#[brw(little)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BehaviourFunctionLabels {
	pub file_name: FileName,

	// pub header_null: u32,
	#[br(temp)]
	#[bw(calc = Header)]
	header: Header,

	pub unknown: u32,
	pub unknown2: u32,

	#[br(temp)]
	#[bw(calc = params.len() as u32)]
	param_count: u32,
	#[br(temp)]
	#[bw(calc = locals.len() as u32)]
	local_count: u32,
	#[br(count = param_count)]
	pub params: Vec<BigString>,
	#[br(count = local_count)]
	pub locals: Vec<BigString>,

	pub unknow3: u32,
	#[br(count = param_count)]
	#[bw(assert(used.len() == param_count as usize))]
	pub used: Vec<u8>,
	pub display_code: u32,
	pub unknown4: u32,
}
