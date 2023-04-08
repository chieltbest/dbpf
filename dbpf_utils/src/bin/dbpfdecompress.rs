use std::env;
use std::ffi::{CStr, OsStr};
use std::io::{Cursor, Read, Seek};
use std::path::{Path, PathBuf};
use dbpf::{CompressionType, DBPFFile, DBPFFile, Index, IndexEntry};
use binrw::{BinRead, Error};
use dbpf::filetypes::DBPFFileType;

fn unpack_header<R: Read + Seek>(header: &mut impl DBPFFile, reader: &mut R, dir_path: &Path) {
    match header.index(reader) {
        Ok(index) => {
            let entries = index.entries();
            for (i, entry) in entries.into_iter().enumerate() {
                let type_id = entry.get_type().clone();
                let group = entry.get_group();
                let instance = entry.get_instance();
                let compression_type = entry.compression_type();

                if let Ok(data) = entry.data(reader) {
                    let raw = data.decompressed().unwrap();
                    let file_basename = if let Some(str) = match type_id {
                        DBPFFileType::Known(t) => {
                            if t.properties().embedded_filename {
                                let name = raw.data.drain(..0x40);
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
                        format!("{:X?}", instance)
                    };
                    let filename = dir_path.join(
                        format!("{i}_{}_{:#8}.{}.{}",
                                file_basename,
                                group,
                                match compression_type {
                                    CompressionType::Streamable => "stream",
                                    CompressionType::Deleted => "deleted",
                                    CompressionType::ZLib => "zlib",
                                    CompressionType::RefPack => "refpack",
                                    CompressionType::Uncompressed => "raw",
                                },
                                type_id.extension()));
                    if let Err(err) = std::fs::write(&filename, &raw.data) {
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

fn unpack_file(mut input: Cursor<Vec<u8>>, path: &Path) {
    let result = DBPFFile::read(&mut input);
    println!("{result:#X?}");

    if let Ok(mut file) = result {
        let dir_path = PathBuf::from(path.file_stem().unwrap_or(OsStr::new("package")));
        if dir_path.is_dir() ||
            std::fs::create_dir(&dir_path)
                .map_err(|e| eprintln!("{}: {e}", dir_path.display()))
                .is_ok() {
            match file {
                DBPFFile::HeaderV1(ref mut h) =>
                    unpack_header(h, &mut input, &dir_path),
                DBPFFile::HeaderV2(ref mut h) =>
                    unpack_header(h, &mut input, &dir_path),
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
