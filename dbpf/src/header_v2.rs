// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

// the bitfield macro will create unused parens, ignore them

use std::io::{Read, Seek, Write};

use binrw::{args, binread, BinRead, BinResult, BinWrite};
use modular_bitfield::{bitfield, prelude::*};

use crate::header_v1::InstanceId;
use crate::{
	dbpf_file::Index,
	filetypes::DBPFFileType,
	internal_file::{CompressionError, FileData, FileDataBinReadArgs},
	lazy_file_ptr::{LazyFilePtr, Zero},
	CompressionType, IndexEntry, IndexMinorVersion,
};
/*#[binread]
#[derive(Clone, Debug)]
pub(crate) struct HeaderV2 {
	#[brw(assert(matches ! (version, Version::V2(_))))]
	pub version: Version,
	pub user_version: UserVersion,
	pub flags: u32,
	pub created: Timestamp,
	pub modified: Timestamp,
	#[br(temp)]
	index_version: u32,
	#[br(temp)]
	index_entry_count: u32,
	#[br(temp)]
	index_location: u32,
	#[br(temp)]
	index_size: u32,
	#[br(temp)]
	hole_index_entry_count: u32,
	#[br(temp)]
	hole_index: u32,
	#[br(temp)]
	hole_index_size: u32,
	#[br(temp)]
	#[bw(calc = 3)]
	index_minor_version: u32,

	#[br(args { inner: args ! { count: index_entry_count as usize }})]
	#[brw(assert(index_entry_count == 0 || index.ptr >= HEADER_SIZE as u64, "index count was {} (non-zero) while index location was {}", index_entry_count, index.ptr))]
	pub index: LazyFilePtr<u64, IndexV2, IndexV2BinReadArgs>,
}*/

#[bitfield]
#[derive(BinRead, BinWrite, Clone, Copy, Debug)]
#[br(map = Self::from_bytes)]
#[bw(map = | & x | Self::into_bytes(x))]
pub struct IndexType {
	fixed_type: bool,
	fixed_group: bool,
	fixed_instance: bool,
	#[skip]
	unused: B29,
}

#[binread]
#[br(import { count: usize })]
#[derive(Clone, Debug)]
pub(crate) struct IndexV2 {
	#[br(temp)]
	index_type: IndexType,

	#[br(temp)]
	#[brw(if (index_type.fixed_type()))]
	type_id: Option<DBPFFileType>,
	#[br(temp)]
	#[brw(if (index_type.fixed_group()))]
	group_id: Option<u32>,
	#[br(temp)]
	#[brw(if (index_type.fixed_instance()))]
	instance_id_ex: Option<u32>,

	#[br(args { count: count, inner: args ! {type_id, group_id, instance_id_ex} })]
	pub entries: Vec<IndexEntryV2>,
}

#[bitfield]
#[derive(BinRead, BinWrite, Clone, Copy, Debug)]
#[br(map = Self::from_bytes)]
#[bw(map = | & x | Self::into_bytes(x))]
pub struct FileSize {
	size: B31,
	ext_compressed: bool,
}

#[binread]
#[brw(import {
type_id: Option < DBPFFileType >,
group_id: Option < u32 >,
instance_id_ex: Option < u32 >})]
#[derive(Clone, Debug)]
pub(crate) struct IndexEntryV2 {
	#[brw(if (type_id.is_none(), type_id.unwrap()))]
	pub type_id: DBPFFileType,
	#[brw(if (group_id.is_none(), group_id.unwrap()))]
	pub group_id: u32,
	#[brw(if (instance_id_ex.is_none(), instance_id_ex.unwrap()))]
	pub instance_id_ex: u32,
	pub instance_id: u32,

	#[br(temp)]
	file_location: u32,
	#[br(temp)]
	file_size: FileSize,
	#[br(temp)]
	decompressed_size: u32,

	#[brw(if (file_size.ext_compressed(), CompressionType::Uncompressed))]
	pub compression_type: CompressionType,
	#[br(temp)]
	#[brw(if (file_size.ext_compressed(), 1))]
	committed: u16,

	#[br(args {
    offset: file_location as u64,
    inner: args ! {
    count: file_size.size() as usize,
    compression_type,
    decompressed_size,
    type_id
    }})]
	pub data: LazyFilePtr<Zero, FileData, FileDataBinReadArgs>,
}

impl Index for IndexV2 {
	fn try_into_vec<R: Read + Seek>(
		self,
		reader: &mut R,
		index_version: IndexMinorVersion,
	) -> BinResult<Vec<IndexEntry>> {
		self.entries
			.into_iter()
			.map(|entry| {
				Ok(IndexEntry {
					type_id: entry.type_id,
					group_id: entry.group_id,
					instance_id: InstanceId {
						id: entry.instance_id as u64 | ((entry.instance_id_ex as u64) << 32),
					},

					compression: entry.compression_type,

					data: entry.data,
				})
			})
			.collect()
	}

	fn write_entries<W: Write + Seek, R: Read + Seek>(
		writer: &mut W,
		reader: &mut R,
		entries: &mut Vec<IndexEntry>,
		index_version: IndexMinorVersion,
	) -> Result<(Vec<u8>, usize), CompressionError> {
		todo!()
	}
}
