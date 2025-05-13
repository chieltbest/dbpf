use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, Weak};
use binrw::BinRead;
use binrw::io::BufReader;
use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};
use egui_dock::egui::TextureOptions;
use lru::LruCache;
use tokio::fs::File;
use tracing::error;
use dbpf::DBPFFile;
use dbpf::internal_file::DecodedFile;
use dbpf::internal_file::resource_collection::ResourceData;
use crate::texture_finder::TextureId;

pub struct ImageCache {
    cache: Arc<Mutex<LruCache<TextureId, TextureHandle>>>,
}


impl ImageCache {
    pub fn new(cap: NonZeroUsize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cap))),
        }
    }

    async fn fetch_texture(cache: Weak<Mutex<LruCache<TextureId, TextureHandle>>>,
                           id: TextureId,
                           ctx: egui::Context) {
        match File::open(&id.path).await {
            Err(err) => {
                error!(?err);
            }
            Ok(async_data) => {
                let data = async_data.into_std().await;
                let mut data = BufReader::new(data);
                let mut file = DBPFFile::read(&mut data).unwrap();
                let index = file.index.iter_mut().find(|entry|
                    entry.type_id == id.tgi.type_id &&
                        entry.group_id == id.tgi.group_id &&
                        entry.instance_id == id.tgi.instance_id).unwrap();
                let idata = index.data(&mut data).unwrap();

                let decoded = idata.decoded().unwrap().unwrap();
                if let DecodedFile::ResourceCollection(res) = decoded {
                    let ResourceData::Texture(tex) = &mut res.entries.first_mut().unwrap().data else { return; };
                    let found_texture = tex.decompress_all().into_iter().find_map(|t| {
                        t.into_iter().rev()
                            .find_map(|mip| mip.ok()
                                .map(|decoded_tex| {
                                    ctx.load_texture(
                                        format!("{:?}", id),
                                        ColorImage::from_rgba_unmultiplied(
                                            [decoded_tex.width, decoded_tex.height],
                                            &decoded_tex.data,
                                        ),
                                        TextureOptions::NEAREST,
                                    )
                                }))
                    });

                    if let Some(tex) = found_texture {
                        let cache = cache.upgrade().unwrap();
                        let mut cw = cache.lock().unwrap();
                        cw.push(id, tex);
                    }
                }
            }
        }
    }

    /// get, and if not found, fetch the texture from disk asynchronously, returning an `egui::TextureHandle`
    pub fn get(&mut self, id: &TextureId, ctx: &egui::Context) -> Option<TextureHandle> {
        let o = self.cache.lock().unwrap().get(id).cloned();
        if o.is_none() {
            // fetch the image concurrently
            tokio::spawn(Self::fetch_texture(Arc::downgrade(&self.cache), id.clone(), ctx.clone()));
        }
        o
    }
}
