use std::ffi::OsStr;
use std::fmt::{Debug, Formatter};
use std::io::{Cursor, Read, Seek};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use binrw::{BinRead, BinResult, BinWrite};
use binrw::io::BufReader;

use futures::{stream, StreamExt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;
use walkdir::WalkDir;
use tokio::fs::File;
use tracing::error;

use dbpf::DBPFFile;
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::filetypes::DBPFFileType::Known;

use dbpf::internal_file::DecodedFile;
use dbpf::internal_file::resource_collection::ResourceData;
use dbpf::internal_file::resource_collection::texture_resource::{TextureFormat, TextureResource};


fn ser_file_type<S: Serializer>(t: &DBPFFileType, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_u32(t.code())
}

fn deser_file_type<'a, D>(d: D) -> Result<DBPFFileType, D::Error> where D: Deserializer<'a> {
    Ok(DBPFFileType::from(u32::deserialize(d)?))
}

pub(crate) fn ser_texture_format<S: Serializer>(t: &TextureFormat, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_u32((*t) as u32)
}

pub(crate) fn deser_texture_format<'a, D>(d: D) -> Result<TextureFormat, D::Error> where D: Deserializer<'a> {
    u32::deserialize(d)
        .and_then(|v| {
            let mut bytes = [0u8; 4];
            v.write_le(&mut Cursor::new(bytes.as_mut_slice())).unwrap();
            TextureFormat::read_le(&mut Cursor::new(bytes.as_mut_slice()))
                .map_err(|err| D::Error::custom(err.to_string()))
        })
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Default, Serialize, Deserialize)]
pub struct TGI {
    #[serde(serialize_with = "ser_file_type", deserialize_with = "deser_file_type")]
    pub type_id: DBPFFileType,
    pub group_id: u32,
    pub instance_id: u64,
}

impl Debug for TGI {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.type_id.properties()
            .map(|prop| prop.name.to_string())
            .unwrap_or_else(|| self.type_id.extension()).as_str())
            .field("group", &self.group_id)
            .field("instance", &self.instance_id)
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Default, Debug, Serialize, Deserialize)]
pub struct TextureId {
    pub path: PathBuf,
    pub tgi: TGI,
}

#[derive(Clone, Eq, PartialEq, Hash, Default, Debug, Serialize, Deserialize)]
pub struct FoundTexture {
    pub id: TextureId,

    pub width: u32,
    pub height: u32,
    #[serde(serialize_with = "ser_texture_format", deserialize_with = "deser_texture_format")]
    pub format: TextureFormat,
    pub mip_levels: u32,

    pub memory_size: usize,
}

fn get_textures<R: Read + Seek>(path: PathBuf, mut header: DBPFFile, reader: &mut R) -> Vec<(TGI, TextureResource)> {
    // TODO proper error handling
    header.index.iter_mut()
        .filter_map(|file| {
            let type_id = file.type_id;
            let group_id = file.group_id;
            let instance_id = file.instance_id;

            if type_id == Known(KnownDBPFFileType::TextureResource) {
                match file.data(reader)
                    .unwrap()
                    .decoded()
                    .inspect_err(|err| {
                        eprintln!("Error in {path:?}: {err:#?}");
                    })
                    .ok()?
                    // .unwrap()
                    .unwrap() {
                    DecodedFile::ResourceCollection(res) =>
                        match &res.entries.first().unwrap().data {
                            ResourceData::Texture(tex) => Some((
                                TGI {
                                    type_id,
                                    group_id,
                                    instance_id,
                                },
                                tex.clone()
                            )),
                            _ => None,
                        }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect()
}

async fn get_path_textures(path: PathBuf) -> (PathBuf, Option<Vec<FoundTexture>>) {
    let data = File::open(&path).await.unwrap().into_std().await;
    let mut data = BufReader::new(data);
    let path_clone = path.clone();
    let result = tokio::task::spawn_blocking(move || -> BinResult<Vec<FoundTexture>> {
        let res = get_textures(path_clone.clone(), DBPFFile::read(&mut data)?, &mut data);
        Ok(res.iter().map(|(tgi, tex)| {
            FoundTexture {
                id: TextureId {
                    path: path_clone.clone(),
                    tgi: *tgi,
                },
                width: tex.width,
                height: tex.height,
                format: tex.format,
                mip_levels: tex.mip_levels,
                memory_size: tex.format.compressed_size(tex.width as usize, tex.height as usize) * tex.textures.len(),
            }
        }).collect())
    }).await.unwrap();
    match result {
        Ok(textures) => {
            (path, Some(textures))
        }
        Err(err) => {
            error!("{err:#?}");
            (path, None)
        }
    }
}

pub async fn find_textures(dirs: Vec<PathBuf>,
                           tx: Sender<FoundTexture>,
                           mut progress: impl FnMut(PathBuf, usize, usize)) {
    let files_futures_vec = dirs.iter().map(|dir| {
        WalkDir::new(dir).sort_by_file_name().into_iter().filter_map(|entry| {
            let path = entry.unwrap().path().to_path_buf();
            if path.extension() == Some(OsStr::new("package")) {
                Some(get_path_textures(path.clone()))
            } else {
                None
            }
        })
    }).flatten().collect::<Vec<_>>();
    let total_files = files_futures_vec.len();
    progress(PathBuf::from(""), 0, total_files);

    let mut tgis_stream = stream::iter(
        files_futures_vec.into_iter()
    ).buffered(512)
        .enumerate();

    while let Some((i, (path, data))) = tgis_stream.next().await {
        progress(path.clone(), i + 1, total_files);
        if let Some(textures) = data {
            textures.into_iter().for_each(|tex| {
                tx.send(tex).unwrap();
            });
        }
    };
}
