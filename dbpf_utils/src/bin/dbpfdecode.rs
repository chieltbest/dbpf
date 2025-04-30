use binrw::{BinRead, Error};
use dbpf::DBPFFile;
use std::env;
use std::ffi::OsStr;
use std::io::{Cursor, Read, Seek};
use std::path::PathBuf;
use walkdir::WalkDir;

fn read_all<R: Read + Seek>(header: &mut DBPFFile, reader: &mut R, path: PathBuf) {
    let num_idx = header.index.len();
    for (i, entry) in header.index.iter_mut().enumerate() {
        match entry.type_id {
            _ => {
                match entry.data(reader) {
                    Err(err) => println!("{err}"),
                    Ok(data) => {
                        match data.decoded() {
                            Err(err) => {
                                println!("{}/{} {:?} {:X} {:X} {:X}: {:?}",
                                         i + 1,
                                         num_idx,
                                         entry.type_id,
                                         entry.type_id.code(),
                                         entry.group_id,
                                         entry.instance_id,
                                         path);
                                println!("{err}")
                            }
                            _ => {}
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
