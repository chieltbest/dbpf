use std::{
	cmp::max,
	ffi::OsStr,
	fmt::{Debug, Formatter},
	io::{Cursor, Read, Seek},
	path::PathBuf,
	sync::mpsc::Sender,
};

use binrw::{io::BufReader, BinRead, BinResult, BinWrite};
use dbpf::{
	filetypes::{DBPFFileType, DBPFFileType::Known, KnownDBPFFileType},
	internal_file::{
		resource_collection::{texture_resource::TextureFormat, ResourceData},
		CompressionError, DecodedFile,
	},
	DBPFFile,
};
use futures::{stream, StreamExt};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use tokio::fs::File;
use tracing::error;
use walkdir::WalkDir;

fn ser_file_type<S: Serializer>(t: &DBPFFileType, ser: S) -> Result<S::Ok, S::Error> {
	ser.serialize_u32(t.code())
}

fn deser_file_type<'a, D>(d: D) -> Result<DBPFFileType, D::Error>
where
	D: Deserializer<'a>,
{
	Ok(DBPFFileType::from(u32::deserialize(d)?))
}

pub(crate) fn ser_texture_format<S: Serializer>(
	t: &TextureFormat,
	ser: S,
) -> Result<S::Ok, S::Error> {
	ser.serialize_u32((*t) as u32)
}

pub(crate) fn deser_texture_format<'a, D>(d: D) -> Result<TextureFormat, D::Error>
where
	D: Deserializer<'a>,
{
	u32::deserialize(d).and_then(|v| {
		let mut bytes = [0u8; 4];
		v.write_le(&mut Cursor::new(bytes.as_mut_slice())).unwrap();
		TextureFormat::read_le(&mut Cursor::new(bytes.as_mut_slice()))
			.map_err(|err| Error::custom(err.to_string()))
	})
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Default, Serialize, Deserialize)]
pub struct Tgi {
	#[serde(serialize_with = "ser_file_type", deserialize_with = "deser_file_type")]
	pub type_id: DBPFFileType,
	pub group_id: u32,
	pub instance_id: u64,
}

impl Debug for Tgi {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(
			self.type_id
				.properties()
				.map(|prop| prop.name.to_string())
				.unwrap_or_else(|| self.type_id.extension())
				.as_str(),
		)
		.field("group", &self.group_id)
		.field("instance", &self.instance_id)
		.finish()
	}
}

#[derive(Clone, Eq, PartialEq, Hash, Default, Debug, Serialize, Deserialize)]
pub struct TextureId {
	pub path: PathBuf,
	pub tgi: Tgi,
}

#[derive(Clone, Eq, PartialEq, Hash, Default, Debug, Serialize, Deserialize)]
pub struct FoundTexture {
	pub id: TextureId,

	pub width: u32,
	pub height: u32,
	#[serde(
		serialize_with = "ser_texture_format",
		deserialize_with = "deser_texture_format"
	)]
	pub format: TextureFormat,
	pub mip_levels: u32,

	pub memory_size: usize,
}

fn get_textures<R: Read + Seek>(
	path: PathBuf,
	header: DBPFFile,
	reader: &mut R,
) -> impl Iterator<Item = FoundTexture> + use<'_, R> {
	header.index.into_iter().filter_map(move |mut file| {
		let type_id = file.type_id;
		let group_id = file.group_id;
		let instance_id = file.instance_id;

		if type_id == Known(KnownDBPFFileType::TextureResource) {
			match file
				.data(reader)
				.map_err(CompressionError::BinResult)
				.and_then(|d| d.decoded())
				.inspect_err(|err| {
					error!(?path, ?err);
				})
				.ok()?
				.expect("TextureResource should always be allowed to decode")
			{
				DecodedFile::ResourceCollection(res) => match &res.entries.first()?.data {
					ResourceData::Texture(tex) => {
						let total_memory = (0..tex.mip_levels())
							.map(|i| {
								tex.get_format().compressed_size(
									max(tex.width as usize >> i, 1),
									max(tex.height as usize >> i, 1),
								) * tex.textures.len()
							})
							.sum();
						Some(FoundTexture {
							id: TextureId {
								path: path.clone(),
								tgi: Tgi {
									type_id,
									group_id,
									instance_id,
								},
							},
							width: tex.width,
							height: tex.height,
							format: tex.get_format(),
							mip_levels: tex.mip_levels() as u32,
							memory_size: total_memory,
						})
					}
					_ => None,
				},
				_ => None,
			}
		} else {
			None
		}
	})
}

async fn get_path_textures(path: PathBuf) -> (PathBuf, Option<Vec<FoundTexture>>) {
	match File::open(&path).await {
		Err(err) => {
			error!(?err);
			(path, None)
		}
		Ok(async_data) => {
			let data = async_data.into_std().await;
			let mut data = BufReader::new(data);
			let path_clone = path.clone();
			let result = tokio::task::spawn_blocking(move || -> BinResult<Vec<FoundTexture>> {
				Ok(
					get_textures(path_clone.clone(), DBPFFile::read(&mut data)?, &mut data)
						.collect(),
				)
			})
			.await
			.unwrap();
			match result {
				Ok(textures) => (path, Some(textures)),
				Err(err) => {
					error!(?err);
					(path, None)
				}
			}
		}
	}
}

pub async fn find_textures(
	dirs: Vec<PathBuf>,
	tx: Sender<FoundTexture>,
	mut progress: impl FnMut(PathBuf, usize, usize),
) {
	let files_futures_vec = dirs
		.iter()
		.flat_map(|dir| {
			WalkDir::new(dir)
				.sort_by_file_name()
				.into_iter()
				.filter_map(|entry| {
					let path = entry
						.inspect_err(|err| error!(?err))
						.ok()?
						.path()
						.to_path_buf();
					if path.extension() == Some(OsStr::new("package")) {
						Some(get_path_textures(path.clone()))
					} else {
						None
					}
				})
		})
		.enumerate()
		.map(|(i, fut)| {
			progress(PathBuf::from(""), 0, i);
			fut
		})
		.collect::<Vec<_>>();
	let total_files = files_futures_vec.len();
	progress(PathBuf::from(""), 0, total_files);

	let mut tgis_stream = stream::iter(files_futures_vec.into_iter())
		.buffered(num_cpus::get())
		.enumerate();

	while let Some((i, (path, data))) = tgis_stream.next().await {
		progress(path.clone(), i + 1, total_files);
		if let Some(textures) = data {
			for tex in textures.into_iter() {
				if tx.send(tex).is_err() {
					return;
				}
			}
		}
	}
}
