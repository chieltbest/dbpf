#![allow(dead_code)]
#![allow(unused_variables)]

// SPDX-FileCopyrightText: 2023-2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod common;
mod dbpf_file;
pub mod filetypes;
pub mod header_v1;
pub mod header_v2;
pub mod internal_file;
mod lazy_file_ptr;

use std::num::TryFromIntError;

use binrw::binrw;
pub use dbpf_file::{DBPFFile, HoleIndexEntry, IndexEntry};

pub const HEADER_SIZE: u32 = 0x60;

#[binrw]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Version {
	#[brw(magic = 1u32)]
	V1(V1Minor),
	#[brw(magic = 2u32)]
	V2(V2Minor),
	#[brw(magic = 3u32)]
	V3(V3Minor),
}

impl Default for Version {
	fn default() -> Self {
		Self::V2(V2Minor::M1)
	}
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum V1Minor {
	M0 = 0,
	M1 = 1,
	M2 = 2,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum V2Minor {
	M0 = 0,
	M1 = 1,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum V3Minor {
	M0 = 0,
}

#[binrw]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct UserVersion {
	pub major: u32,
	pub minor: u32,
}

#[binrw]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default)]
pub struct Timestamp(pub u32);

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum IndexVersion {
	#[default]
	Default = 7,
	Spore = 0,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub enum IndexMinorVersion {
	V0 = 0,
	V1 = 1,
	#[default]
	V2 = 2,
	V3 = 3,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum CompressionType {
	#[default]
	Uncompressed = 0x0000,
	Streamable = 0xFFFE,
	RefPack = 0xFFFF,
	Deleted = 0xFFE0,
	ZLib = 0x5A42,
}

#[derive(Debug)]
pub enum DBPFError {
	FixedType,
	FixedGroup,
	FixedInstance,

	BadInt(TryFromIntError),
	BadFormat(binrw::Error),
}
