// SPDX-FileCopyrightText: 2023-2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Read, Seek, SeekFrom, Write};

use binrw::{binread, binrw, parser, BinRead, BinResult, BinWrite};

use crate::{
	filetypes::DBPFFileType,
	header_v1::{IndexV1, IndexV1BinReadArgs},
	header_v2::{IndexV2, IndexV2BinReadArgs},
	internal_file::{CompressionError, FileData, FileDataBinReadArgs},
	lazy_file_ptr::{LazyFilePtr, Zero},
	CompressionType, IndexMinorVersion, IndexVersion, Timestamp, UserVersion, Version, HEADER_SIZE,
};

#[binrw]
#[brw(magic = b"DBPF", little)]
#[derive(Clone, Debug, Default)]
pub struct DBPFHeader {
	pub version: Version,
	pub user_version: UserVersion,
	pub flags: u32,
	pub created: Timestamp,
	pub modified: Timestamp,
	pub index_version: IndexVersion,

	pub index_entry_count: u32,

	pub(crate) index_location: u32,

	pub(crate) index_size: u32,

	pub(crate) hole_index_entry_count: u32,

	pub(crate) hole_index_location: u32,

	#[br(temp)]
	#[bw(calc = 0)]
	hole_index_size: u32,

	pub index_minor_version: IndexMinorVersion,

	pub(crate) index_offset: u64,
}

#[binread]
#[br(little)]
#[derive(Clone, Debug, Default)]
pub struct DBPFFile {
	pub header: DBPFHeader,

	#[br(
        seek_before = SeekFrom::Start(header.hole_index_location as u64), count = header.hole_index_entry_count
    )]
	pub hole_index: Vec<HoleIndexEntry>,

	#[br(seek_before = SeekFrom::Start(if matches!(header.version, Version::V1(_)) { header.index_location as u64 } else { header.index_offset }))]
	#[br(
        parse_with = parse_index, args ( header.index_entry_count, header.version, header.index_minor_version )
    )]
	pub index: Vec<IndexEntry>,
}

#[parser(reader: r)]
fn parse_index(
	count: u32,
	version: Version,
	index_version: IndexMinorVersion,
) -> BinResult<Vec<IndexEntry>> {
	match version {
		Version::V1(_) => IndexV1::read_le_args(
			r,
			IndexV1BinReadArgs::builder()
				.count(count as usize)
				.version(index_version)
				.finalize(),
		)?
		.try_into_vec(r, index_version),
		Version::V2(_) | Version::V3(_) => IndexV2::read_le_args(
			r,
			IndexV2BinReadArgs::builder()
				.count(count as usize)
				.finalize(),
		)?
		.try_into_vec(r, index_version),
	}
}

#[derive(Clone, Debug)]
pub struct IndexEntry {
	pub type_id: DBPFFileType,
	pub group_id: u32,
	pub instance_id: u64,

	pub compression: CompressionType,
	pub(crate) data: LazyFilePtr<Zero, FileData, FileDataBinReadArgs>,
}

impl IndexEntry {
	pub fn data<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<&mut FileData> {
		self.data.get(reader)
	}
}

#[binrw]
#[derive(Copy, Clone, Debug, Default)]
pub struct HoleIndexEntry {
	pub location: u32,
	pub size: u32,
}

pub trait Index {
	fn try_into_vec<R: Read + Seek>(
		self,
		reader: &mut R,
		index_version: IndexMinorVersion,
	) -> BinResult<Vec<IndexEntry>>;

	/// write all index entries to a file, returning the index that is to be written directly after the header
	fn write_entries<W: Write + Seek, R: Read + Seek>(
		writer: &mut W,
		reader: &mut R,
		entries: &mut Vec<IndexEntry>,
		index_version: IndexMinorVersion,
	) -> Result<(Vec<u8>, usize), CompressionError>;
}

impl DBPFFile {
	pub fn write<W: Write + Seek, R: Read + Seek>(
		&mut self,
		writer: &mut W,
		reader: &mut R,
	) -> Result<(), CompressionError> {
		writer
			.seek(SeekFrom::Start(HEADER_SIZE as u64))
			.map_err(binrw::Error::from)?;
		let (index, count) = match self.header.version {
			Version::V1(_) => IndexV1::write_entries(
				writer,
				reader,
				&mut self.index,
				self.header.index_minor_version,
			),
			Version::V2(_) | Version::V3(_) => todo!(),
		}?;
		self.header.index_size = index.len() as u32;
		self.header.index_location = HEADER_SIZE;
		self.header.index_entry_count = count as u32;
		self.header.hole_index_entry_count = 0;
		self.header.hole_index_location = 0;
		writer
			.seek(SeekFrom::Start(0))
			.map_err(binrw::Error::from)?;
		self.header.write(writer)?;
		writer
			.seek(SeekFrom::Start(HEADER_SIZE as u64))
			.map_err(binrw::Error::from)?;
		index.write(writer)?;
		Ok(())
	}
}
