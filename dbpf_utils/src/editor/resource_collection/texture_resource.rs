use crate::editor::Editor;
use dbpf::internal_file::resource_collection::texture_resource::{DecodedTexture, TextureFormat, TextureResource, TextureResourceData};
use eframe::egui;
use eframe::egui::{Button, ColorImage, ComboBox, DragValue, Response, TextureOptions, Ui};
use image::ImageReader;
use std::cmp::min;
use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use tracing::error;

#[derive(Default)]
pub struct TextureResourceEditorState {
    textures: Vec<Vec<Option<egui::TextureHandle>>>,
    zoom_state: Vec<(egui::Rect, usize)>,
    original_texture_bgra: TextureResource,
}

impl Debug for TextureResourceEditorState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.textures.iter().map(|texture| {
                texture.iter().map(|mip| {
                    mip.as_ref().map(|img| {
                        format!("{}: {:?}", img.name(), img.size_vec2())
                    })
                }).collect::<Vec<_>>()
            }))
            .finish()
    }
}

impl TextureResourceEditorState {
    fn load_textures(res: &TextureResource, context: &egui::Context) -> Vec<Vec<Option<egui::TextureHandle>>> {
        res.decompress_all().into_iter().enumerate().map(|(tex_num, texture)| {
            texture.into_iter().enumerate().rev().map(|(mip_num, mip)| {
                mip.inspect_err(|err| error!(?err))
                    .ok().map(|decoded| {
                    context.load_texture(
                        format!("texture{tex_num}_mip{mip_num}"),
                        ColorImage::from_rgba_unmultiplied(
                            [decoded.width, decoded.height],
                            &decoded.data),
                        TextureOptions::NEAREST,
                    )
                })
            }).collect()
        }).collect()
    }

    fn refresh_textures_from(&mut self, res: &TextureResource, context: &egui::Context) {
        self.textures = Self::load_textures(res, context);
        self.zoom_state.resize(self.textures.len(), (egui::Rect::ZERO, 0));
        let mip_levels = res.mip_levels() - 1;
        self.zoom_state.iter_mut()
            .for_each(|(_r, mip)| *mip = min(*mip, mip_levels))
    }
}

impl Editor for TextureResource {
    type EditorState = TextureResourceEditorState;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState {
        let mut new = Self::EditorState {
            original_texture_bgra: self.recompress_with_format(TextureFormat::RawBGRA).unwrap(),
            ..Default::default()
        };
        new.refresh_textures_from(self, context);
        new
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut update_images = false;

        let mut res = self.file_name.name.show_editor(&mut (), ui);
        ui.horizontal_wrapped(|ui| {
            res |= ui.add_enabled(false, DragValue::new(&mut self.width));
            ui.label("width");
            res |= ui.add_enabled(false, DragValue::new(&mut self.height));
            ui.label("height");
        });
        ui.horizontal(|ui| {
            res |= ui.add_enabled(false, DragValue::new(
                &mut self.textures.first()
                    .map(|t| t.entries.len())
                    .unwrap_or(0)));
            ui.label("Mip levels");

            let top_is_lifo = self.textures.iter().any(|tex| {
                matches!(tex.entries.last(), Some(TextureResourceData::LIFOFile { .. }))
            });
            let bottom_is_lifo = self.textures.iter().any(|tex| {
                matches!(tex.entries.first(), Some(TextureResourceData::LIFOFile { .. }))
            });

            if ui.add_enabled(!top_is_lifo,
                              Button::new("Recalculate all mipmaps"))
                .clicked() {
                state.original_texture_bgra.remove_smaller_mip_levels();
                state.original_texture_bgra.add_max_mip_levels();
                res.mark_changed();
                update_images = true;
            }
            if ui.add_enabled(self.mip_levels() < self.max_mip_levels() && !bottom_is_lifo,
                              Button::new("Add missing mipmaps"))
                .clicked() {
                state.original_texture_bgra.add_max_mip_levels();
                res.mark_changed();
                update_images = true;
            }
            if ui.add_enabled(self.mip_levels() > 1,
                              Button::new("Remove all mipmaps"))
                .clicked() {
                state.original_texture_bgra.remove_smaller_mip_levels();
                res.mark_changed();
                update_images = true;
            }
            if ui.add_enabled(self.mip_levels() > 1,
                              Button::new("Remove largest texture"))
                .on_hover_text("Deletes the biggest image from the list of mipmaps, effectively halving the image size")
                .clicked() {
                state.original_texture_bgra.remove_largest_mip_levels(1);
                res.mark_changed();
                update_images = true;
            }
        });

        let formats = [
            TextureFormat::DXT5,
            TextureFormat::DXT3,
            TextureFormat::DXT1,
            TextureFormat::Alpha,
            TextureFormat::Grayscale,
            TextureFormat::RawBGRA,
            TextureFormat::RawBGR,
            TextureFormat::AltBGRA,
            TextureFormat::AltBGR,
        ];
        let mut current_format = self.get_format();
        let prev_format = current_format;
        ComboBox::new("format", "Texture Format")
            .selected_text(format!("{:?}", current_format))
            .show_ui(ui, |ui| {
                for format in formats {
                    ui.selectable_value(&mut current_format, format, format!("{:?}", format));
                }
            });
        if current_format != prev_format {
            update_images = true;
        }

        if update_images {
            if let Ok(mut new) = state.original_texture_bgra.recompress_with_format(current_format) {
                new.file_name = std::mem::take(&mut self.file_name);
                new.file_name_repeat = std::mem::take(&mut self.file_name_repeat);
                new.purpose = self.purpose;
                new.unknown = self.unknown;
                *self = new;
                state.refresh_textures_from(self, ui.ctx());
            }
            update_images = false;
        }

        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.purpose));
            ui.radio_value(&mut self.purpose, 1.0, "Object");
            ui.radio_value(&mut self.purpose, 2.0, "Outfit");
            ui.radio_value(&mut self.purpose, 3.0, "Interface");
        });

        let mip_level_names = (0..self.mip_levels()).map(|mip| {
            let (w, h) = self.mip_size(mip);
            format!("{w}x{h}")
        }).enumerate().collect::<Vec<_>>();
        let original_size = self.mip_size(0);

        for (texture_num, (texture, (zoom, cur_selected_mip_level))) in self.textures.iter_mut()
            .zip(&mut state.zoom_state)
            .enumerate() {
            ui.separator();

            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut texture.creator_id)
                    .hexadecimal(8, false, true))
                    | ui.label("Creator ID")
            }).inner.on_hover_text("Creator ID of the creator of this texture\n\
                    If the texture has not been uploaded to online services the creator ID will be either \
                    FF000000 or FFFFFFFF");

            ui.horizontal(|ui| {
                if ui.button("Reset zoom").clicked() {
                    *zoom = egui::Rect::ZERO;
                }
                for (mip_i, name) in mip_level_names.clone() {
                    ui.radio_value(cur_selected_mip_level, mip_i, name);
                }
            });

            egui::Frame::canvas(ui.style())
                .show(ui, |ui| {
                    egui::Scene::new()
                        .zoom_range(0.1..=16.0)
                        .show(ui, zoom, |ui| {
                            if let Some(mip_level) = texture.entries.get(*cur_selected_mip_level) {
                                match mip_level {
                                    TextureResourceData::Embedded(_) => {
                                        if let Some(image) = state.textures[texture_num][*cur_selected_mip_level].as_ref() {
                                            ui.add(egui::Image::new(image)
                                                .fit_to_exact_size(
                                                    egui::Vec2::new(
                                                        original_size.0 as f32,
                                                        original_size.1 as f32)));
                                        }
                                    }
                                    TextureResourceData::LIFOFile { file_name } => {
                                        ui.end_row();
                                        ui.label(format!("file: {}",
                                                         String::from_utf8_lossy(&file_name.data)));
                                    }
                                }
                            }
                        });
                });
        }

        ui.input_mut(|i| {
            if let Some(file) = i.raw.dropped_files.first().cloned() {
                let image = if let Some(path) = file.path {
                    ImageReader::open(path)
                        .ok().map(|i| i.decode())
                } else {
                    file.bytes.and_then(|bytes|
                        ImageReader::new(Cursor::new(bytes))
                            .with_guessed_format()
                            .ok())
                        .map(|i| i.decode())
                };

                if let Some(Ok(image)) = image {
                    let orig_format = self.get_format();

                    let has_mip = self.mip_levels() > 1;

                    self.compress_replace(DecodedTexture {
                        width: image.width() as usize,
                        height: image.height() as usize,
                        data: image.into_rgba8().to_vec(),
                    }, Some(TextureFormat::RawBGRA));

                    state.original_texture_bgra = self.clone();

                    if has_mip {
                        self.add_max_mip_levels();
                    }
                    if let Ok(new) = self.recompress_with_format(orig_format) {
                        *self = new;
                    }

                    update_images = true;

                    i.raw.dropped_files.clear();
                }
            }
        });

        if update_images {
            state.refresh_textures_from(&self, ui.ctx());
        }

        res
    }
}
