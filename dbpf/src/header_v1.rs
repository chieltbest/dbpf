use std::{
	collections::HashMap,
	io::{Cursor, Read, Seek, SeekFrom, Write},
	num::NonZeroU32,
};

use binrw::{args, binread, binrw, BinRead, BinResult, BinWrite, BinWriterExt, Endian, Error};

use crate::{
	dbpf_file::Index,
	filetypes::{DBPFFileType, KnownDBPFFileType},
	internal_file::{
		dbpf_directory::{DBPFDirectory, DBPFDirectoryBinWriteArgs, DBPFDirectoryEntry},
		CompressedFileData, CompressionError, FileData, FileDataBinReadArgs, FileDataInternal,
	},
	lazy_file_ptr::{LazyFilePtr, Zero},
	CompressionType, IndexEntry, IndexMinorVersion,
};

#[binread]
#[br(import { count: usize, version: IndexMinorVersion })]
#[derive(Clone, Debug)]
pub(crate) struct IndexV1 {
	#[br(args { count: count, inner: args ! { version: version } })]
	pub entries: Vec<IndexEntryV1>,
}

#[binrw]
#[brw(import { version: IndexMinorVersion })]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Debug)]
pub struct InstanceId {
	#[br(temp)]
	#[bw(calc(* id as u32))]
	id_lower: u32,
	#[br(temp)]
	#[bw(calc((id >> 32) as u32))]
	#[brw(if (matches ! (version, IndexMinorVersion::V2)))]
	id_upper: u32,
	#[br(calc(id_lower as u64 | ((id_upper as u64) << 32)))]
	#[bw(ignore)]
	pub id: u64,
}

#[binread]
#[brw(import { version: IndexMinorVersion })]
#[derive(Clone, Debug)]
pub(crate) struct IndexEntryV1 {
	pub type_id: DBPFFileType,
	pub group_id: u32,
	#[brw(args { version: version })]
	pub instance_id: InstanceId,
	#[br(temp)]
	location: NonZeroU32,
	#[br(temp)]
	size: u32,

	#[brw(ignore)]
	pub compression: Option<CompressionType>,
	#[br(args {
    offset: u32::from(location) as u64,
    inner: args ! {
    count: size as usize,
    compression_type: CompressionType::Uncompressed,
    decompressed_size: size,
    type_id
    }})]
	pub data: LazyFilePtr<Zero, FileData, FileDataBinReadArgs>,
}

impl Index for IndexV1 {
	fn try_into_vec<R: Read + Seek>(
		mut self,
		reader: &mut R,
		index_version: IndexMinorVersion,
	) -> BinResult<Vec<IndexEntry>> {
		let mut compressed_entries = HashMap::new();

		self.entries
			.iter_mut()
			.try_for_each(|entry| match entry.type_id {
				DBPFFileType::Known(KnownDBPFFileType::DBPFDirectory) => {
					let data = entry.data.get(reader)?;
					let raw_data = &mut data
						.decompressed()
						.expect("Uncompressed data decompress is infallible")
						.data;
					let res: DBPFDirectory =
						DBPFDirectory::read_args(&mut Cursor::new(raw_data), args! {
							version: index_version
						})?;
					for entry in res.entries {
						compressed_entries.insert(
							(entry.type_id, entry.group_id, entry.instance_id),
							entry.decompressed_size,
						);
					}
					Ok::<_, Error>(())
				}
				_ => Ok(()),
			})?;

		self.entries
			.into_iter()
			.filter_map(|entry| {
				if matches!(
					entry.type_id,
					DBPFFileType::Known(KnownDBPFFileType::DBPFDirectory)
				) {
					return None;
				}

				let mut compression = CompressionType::Uncompressed;
				let mut data = entry.data;

				if let Some(decompressed_size) =
					compressed_entries.get(&(entry.type_id, entry.group_id, entry.instance_id))
				{
					compression = CompressionType::RefPack;

					data.args.inner.compression_type = CompressionType::RefPack;
					data.args.inner.decompressed_size = *decompressed_size;
				}

				Some(Ok(IndexEntry {
					type_id: entry.type_id,
					group_id: entry.group_id,
					instance_id: entry.instance_id.id,
					compression,
					data,
				}))
			})
			.collect()
	}

	fn write_entries<W: Write + Seek, R: Read + Seek>(
		writer: &mut W,
		reader: &mut R,
		entries: &mut Vec<IndexEntry>,
		index_version: IndexMinorVersion,
	) -> Result<(Vec<u8>, usize), CompressionError> {
		let mut index_buf = Cursor::new(Vec::new());

		let has_compressed = entries
			.iter()
			.any(|e| matches!(e.compression, CompressionType::RefPack));
		let num_entries = entries.len() + if has_compressed { 1 } else { 0 };

		let index_size = num_entries
			* match index_version {
				IndexMinorVersion::V0 | IndexMinorVersion::V1 => 20,
				IndexMinorVersion::V2 => 24,
				IndexMinorVersion::V3 => {
					return Err(Error::AssertFail {
						message: "Header v1 does not support index minor v3".to_string(),
						pos: 0,
					}
					.into())
				}
			};
		writer
			.seek(SeekFrom::Current(index_size as i64))
			.map_err(Error::from)?;

		let dir = DBPFDirectory {
			entries: entries
				.iter_mut()
				.filter_map(|entry| match entry.compression {
					CompressionType::Uncompressed => None,
					_ => Some((|| match entry.compression {
						CompressionType::RefPack => Ok(DBPFDirectoryEntry {
							type_id: entry.type_id,
							group_id: entry.group_id,
							instance_id: InstanceId {
								id: entry.instance_id,
							},
							decompressed_size: entry
								.data(reader)?
								.compressed(CompressionType::RefPack)?
								.decompressed_size,
						}),
						CompressionType::Streamable
						| CompressionType::Deleted
						| CompressionType::ZLib => Err(CompressionError::from(Error::AssertFail {
							message: format!(
								"Unsupported compression type found in header v1 DBPF file: {:?}",
								entry.compression
							),
							pos: 0,
						})),
						_ => unreachable!(),
					})()),
				})
				.collect::<Result<Vec<_>, CompressionError>>()?,
		};

		if has_compressed {
			let mut dir_buf = Cursor::new(Vec::new());
			dir_buf.write_le_args(
				&dir,
				DBPFDirectoryBinWriteArgs::builder()
					.version(index_version)
					.finalize(),
			)?;

			let dir_entry = IndexEntry {
				type_id: DBPFFileType::Known(KnownDBPFFileType::DBPFDirectory),
				group_id: 0xE86B1EEF,
				instance_id: 0x286B1F03,

				compression: CompressionType::Uncompressed,

				data: LazyFilePtr::from_data(
					FileData {
						type_id: DBPFFileType::Known(KnownDBPFFileType::DBPFDirectory),
						data: FileDataInternal::Compressed(CompressedFileData {
							compression_type: CompressionType::Uncompressed,
							decompressed_size: dir_buf.position() as u32,
							data: dir_buf.into_inner(),
						}),
					},
					Endian::Little,
					FileDataBinReadArgs::builder()
						.count(0)
						.compression_type(CompressionType::Uncompressed)
						.decompressed_size(0)
						.type_id(DBPFFileType::Known(KnownDBPFFileType::DBPFDirectory))
						.finalize(),
				),
			};

			entries.insert(0, dir_entry);
		}

		for entry in &mut *entries {
			entry.type_id.write_le(&mut index_buf)?;
			entry.group_id.write_le(&mut index_buf)?;
			InstanceId {
				id: entry.instance_id,
			}
			.write_le_args(&mut index_buf, args! {
				version: index_version
			})?;

			let location = writer.stream_position().map_err(Error::from)? as u32;
			// this will cause all entries in the index to be opened, maybe do a clone?
			let compression = entry.compression;
			let data = entry.data(reader)?;
			let compressed = data.compressed(compression)?;

			compressed.write_le(writer)?;

			let size = writer.stream_position().map_err(Error::from)? as u32 - location;

			location.write_le(&mut index_buf)?;
			size.write_le(&mut index_buf)?;
		}

		if has_compressed {
			entries.remove(0);
		}

		Ok((index_buf.into_inner(), num_entries))
	}
}
