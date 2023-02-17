#![feature(cstr_from_bytes_until_nul)]

use std::env;
use std::ffi::{CStr, OsStr};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use dbpf::DBPFFile;
use binrw::{BinRead, Error};
use refpack::format::TheSims12;
use dbpf::filetypes::DBPFFileType;

fn unpack_file(mut input: Cursor<Vec<u8>>, path: &Path) {
    let result = DBPFFile::read(&mut input);
    println!("{result:#X?}");

    if let Ok(mut file) = result {
        let dir_path = PathBuf::from(path.file_stem().unwrap_or(OsStr::new("package")));
        if dir_path.is_dir() ||
            std::fs::create_dir(&dir_path)
                .map_err(|e| eprintln!("{}: {e}", dir_path.display()))
                .is_ok() {
            match file.header.index.get(&mut input) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(data) = entry.data.get(&mut input) {
                            let mut raw = data.data.clone();
                            let mut compressed = false;
                            if let Ok(data)
                                = refpack::easy_decompress::<TheSims12>(raw.as_slice()) {
                                compressed = true;
                                raw = data;
                            }
                            let file_basename = if let Some(str) = match entry.type_id {
                                DBPFFileType::Known(t) => {
                                    if t.properties().embedded_filename {
                                        let name = raw.drain(..0x40);
                                        let str = CStr::from_bytes_until_nul(name.as_slice())
                                            .unwrap().to_str().unwrap().to_string();
                                        Some(str)
                                    } else {
                                        None
                                    }
                                }
                                _ => None
                            } {
                                str.to_string()
                            } else {
                                format!("0x{:X?}", entry.instance_id.id)
                            };
                            let filename = dir_path.join(
                                format!("{}.{}.{}",
                                        file_basename,
                                        if compressed { "refpak" } else { "raw" },
                                        entry.type_id.extension()));
                            if let Err(err) = std::fs::write(&filename, &raw) {
                                eprintln!("{}: {err}", &filename.display());
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("{err}");
                }
            }
        }
    }
}

fn main() -> Result<(), Error> {
    for arg in env::args_os().skip(1) {
        let path = Path::new(&arg);
        let input = Cursor::new(std::fs::read(path).unwrap());
        unpack_file(input, path);
    }
    Ok(())
}
