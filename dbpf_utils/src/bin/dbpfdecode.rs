use std::env;
use std::io::{Cursor, Read, Seek};
use dbpf::{DBPFFile, Header, Index, IndexEntry};
use binrw::{BinRead, Error};

fn read_all<R: Read + Seek>(header: &mut impl Header, reader: &mut R) {
    match header.index(reader) {
        Ok(index) => {
            for entry in index.entries() {
                match entry.data(reader) {
                    Err(err) => eprintln!("{err}"),
                    Ok(data) => {
                        match data.decoded() {
                            Some(Err(err)) => eprintln!("{err:?}"),
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
        let mut input = Cursor::new(std::fs::read(arg.clone()).unwrap());

        if env::args_os().len() > 2 {
            println!("{arg:?}");
        }
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
    }
    Ok(())
}
