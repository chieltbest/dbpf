use std::env;
use std::ffi::OsStr;
use std::io::{Cursor, Read, Seek};
use std::os::unix::ffi::OsStrExt;
use dbpf::{DBPFFile, DBPFFile, Index, IndexEntry};
use binrw::{BinRead, Error};
use walkdir::WalkDir;

fn read_all<R: Read + Seek>(header: &mut impl DBPFFile, reader: &mut R) {
    match header.index(reader) {
        Ok(index) => {
            for (i, entry) in index.entries().into_iter().enumerate() {
                // println!("{i} {:?} {:X} {:X}", entry.get_type(), entry.get_group(), entry.get_instance());
                match entry.data(reader) {
                    Err(err) => eprintln!("{err}"),
                    Ok(data) => {
                        match data.decompressed() {
                            Err(err) => eprintln!("{err}"),
                            _ => {}
                        }
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }
}

fn main() -> Result<(), Error> {
    for arg in env::args_os().skip(1) {
        WalkDir::new(arg).into_iter().filter_map(|f| {
            f.ok().and_then(|e| {
                if e.file_type().is_file() &&
                    e.path().extension() == Some(OsStr::from_bytes("package".as_bytes())) {
                    Some(e.path().to_path_buf())
                } else {
                    None
                }
            })
        }).for_each(|path| {
            let mut input = Cursor::new(std::fs::read(path.clone()).unwrap());

            println!("{path:?}");
            let mut file = DBPFFile::read(&mut input);
            match file {
                Ok(DBPFFile::HeaderV1(ref mut header)) => {
                    read_all(header, &mut input);
                    if let Err(err) = header.hole_index.get(&mut input) {
                        eprintln!("{err}");
                    }
                }
                Ok(DBPFFile::HeaderV2(ref mut header)) => read_all(header, &mut input),
                _ => {}
            }
            println!("{:#X?}", file);
        });
    }
    Ok(())
}
