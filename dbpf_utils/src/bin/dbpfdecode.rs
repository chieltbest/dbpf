use std::{
	env,
	ffi::OsStr,
	io::{Cursor, Read, Seek},
	path::PathBuf,
};

use binrw::{BinRead, Error};
use dbpf::{
	filetypes::{DBPFFileType, KnownDBPFFileType},
	DBPFFile,
};
use walkdir::WalkDir;

fn print_path(i: usize, i_total: usize, tid: DBPFFileType, gid: u32, iid: u64, path: PathBuf) {
	println!(
		"{}/{} {:?} {:X} {:X} {:X}: {:?}",
		i + 1,
		i_total,
		tid,
		tid.code(),
		gid,
		iid,
		path
	);
}

fn read_all<R: Read + Seek>(header: &mut DBPFFile, reader: &mut R, path: PathBuf) {
	let num_idx = header.index.len();
	for (i, entry) in header.index.iter_mut().enumerate() {
		let tid = entry.type_id;
		let gid = entry.group_id;
		let iid = entry.instance_id;
		match (tid, gid, iid) {
			// known bad resources
			(
				DBPFFileType::Known(KnownDBPFFileType::TrackSettings),
				0x0DA1F2CA,
				0xDDB5D85EFF99E0DE,
			)
			| (
				DBPFFileType::Known(KnownDBPFFileType::TrackSettings),
				0xEB8AB356,
				0x12D2658DFF8BCEB2,
			)
			| (DBPFFileType::Known(KnownDBPFFileType::WallXML), 0x4C8CC5C0, 0x0CE4B4DA)
			| (DBPFFileType::Known(KnownDBPFFileType::WallXML), 0x4C8CC5C0, 0xCCC26AC8)
			| (DBPFFileType::Known(KnownDBPFFileType::WallXML), 0x4C8CC5C0, 0x2CE48516) => {}
			_ => match entry.data(reader) {
				Err(err) => println!("{err}"),
				Ok(data) => {
					let orig_decompressed = data.decompressed().cloned();
					match data.decoded() {
						Ok(Some(orig_decoded)) => {
							let orig_decoded = orig_decoded.clone();
							let new_decompressed = data.decompressed().cloned();
							let new_decoded = data.decoded();
							match (orig_decoded, new_decoded) {
								(_, Err(err)) => {
									if let (Ok(old), Ok(new)) =
										(orig_decompressed, new_decompressed)
									{
										println!(
											"{}",
											similar::TextDiff::from_lines(
												&format!("{old:?}"),
												&format!("{new:?}")
											)
											.unified_diff()
											.header("old", "new")
										);
									}
									print_path(i, num_idx, tid, gid, iid, path.clone());
									println!("{err}");
									println!("Could not decode data after writing decoded!");
									println!();
								}
								(d1, Ok(Some(d2))) => {
									if d1 != *d2 {
										println!("{d1:?}");
										println!("{d2:?}");
										println!("data was not the same!");
										print_path(i, num_idx, tid, gid, iid, path.clone());
									}
								}
								_ => {}
							}
						}
						Err(err) => {
							if let Ok(data) = data.decompressed() {
								println!("{data:?}");
							}
							print_path(i, num_idx, tid, gid, iid, path.clone());
							println!("{err}");
							println!();
						}
						_ => {}
					}
				}
			},
		}
	}
}

fn main() -> Result<(), Error> {
	for arg in env::args_os().skip(1) {
		WalkDir::new(arg)
			.into_iter()
			.filter_map(|f| {
				f.ok().and_then(|e| {
					if e.file_type().is_file()
						&& (e.path().extension() == Some(OsStr::new("package"))
							|| e.path().extension() == Some(OsStr::new("dat")))
					{
						Some(e.path().to_path_buf())
					} else {
						None
					}
				})
			})
			.for_each(|path| {
				let mut input = Cursor::new(std::fs::read(path.clone()).unwrap());

				if let Ok(mut file) = DBPFFile::read(&mut input) {
					read_all(&mut file, &mut input, path);
				}
			});
	}
	Ok(())
}
