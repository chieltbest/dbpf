use std::env;
use std::ffi::OsStr;
use std::io::{Cursor, Read, Seek};
use dbpf::DBPFFile;
use binrw::{BinRead, Error};
use walkdir::WalkDir;

fn read_all<R: Read + Seek>(header: &mut DBPFFile, reader: &mut R) {
    for (i, entry) in header.index.iter_mut().enumerate() {
        println!("{i} {:?} {:X} {:X}", entry.type_id, entry.group_id, entry.instance_id);
        match entry.data(reader) {
            Err(err) => println!("{err}"),
            Ok(data) => {
                match data.decompressed() {
                    Err(err) => println!("{err}"),
                    _ => {}
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

            println!("{path:?}");
            let mut file = DBPFFile::read(&mut input).unwrap();
            read_all(&mut file, &mut input);
            // println!("{:#X?}", file);
        });
    }
    Ok(())
}
