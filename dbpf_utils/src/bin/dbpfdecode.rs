use binrw::{BinRead, Error};
use dbpf::DBPFFile;
use std::env;
use std::ffi::OsStr;
use std::io::{Cursor, Read, Seek, Write};
use std::path::PathBuf;
use walkdir::WalkDir;
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};

fn read_all<R: Read + Seek>(header: &mut DBPFFile, reader: &mut R, path: PathBuf) {
    let num_idx = header.index.len();
    for (i, entry) in header.index.iter_mut().enumerate() {
        match (entry.type_id, entry.group_id, entry.instance_id) {
            // known bad resources
            (DBPFFileType::Known(KnownDBPFFileType::TrackSettings), 0x0DA1F2CA, 0xDDB5D85EFF99E0DE) |
            (DBPFFileType::Known(KnownDBPFFileType::TrackSettings), 0xEB8AB356, 0x12D2658DFF8BCEB2) |
            (DBPFFileType::Known(KnownDBPFFileType::WallXML), 0x4C8CC5C0, 0x0CE4B4DA) |
            (DBPFFileType::Known(KnownDBPFFileType::WallXML), 0x4C8CC5C0, 0xCCC26AC8) |
            (DBPFFileType::Known(KnownDBPFFileType::WallXML), 0x4C8CC5C0, 0x2CE48516) => {}
            _ => {
                match entry.data(reader) {
                    Err(err) => println!("{err}"),
                    Ok(data) => {
                        if let Err(err) = data.decoded() {
                            if let Ok(data) = data.decompressed() {
                                std::io::stdout().write_all(&data.data).unwrap();
                                println!();
                            }
                            println!("{}/{} {:?} {:X} {:X} {:X}: {:?}",
                                     i + 1,
                                     num_idx,
                                     entry.type_id,
                                     entry.type_id.code(),
                                     entry.group_id,
                                     entry.instance_id,
                                     path);
                            println!("{err}");
                            println!();
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Error> {
    for arg in env::args_os().skip(1) {
        WalkDir::new(arg).into_iter().filter_map(|f| {
            f.ok().and_then(|e| {
                if e.file_type().is_file() &&
                    (e.path().extension() == Some(OsStr::new("package")) ||
                        e.path().extension() == Some(OsStr::new("dat"))) {
                    Some(e.path().to_path_buf())
                } else {
                    None
                }
            })
        }).for_each(|path| {
            let mut input = Cursor::new(std::fs::read(path.clone()).unwrap());

            if let Ok(mut file) = DBPFFile::read(&mut input) {
                read_all(&mut file, &mut input, path);
            }
        });
    }
    Ok(())
}
