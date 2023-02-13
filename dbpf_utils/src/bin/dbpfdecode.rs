use std::env;
use std::io::{Cursor};
use dbpf::DBPFFile;
use binrw::{BinRead, Error};

fn main() -> Result<(), Error> {
    for arg in env::args_os().skip(1) {
        let mut input = Cursor::new(std::fs::read(arg).unwrap());

        let file = DBPFFile::read(&mut input);
        if let Err(err) = file {
            eprintln!("{err}");
            continue;
        }
        let mut file = file.unwrap();

        match file.header.index.get(&mut input) {
            Ok(index) => {
                for entry in index {
                    if let Err(err) = entry.data.get(&mut input) {
                        eprintln!("{err}");
                    }
                }
            }
            Err(err) => {
                eprintln!("{err}");
            }
        }

        if let Err(err) = file.header.hole_index.get(&mut input) {
            eprintln!("{err}");
            continue;
        }

        println!("{:#X?}", file);
    }
    Ok(())
}
