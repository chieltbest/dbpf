use std::fmt::{Debug, Formatter};
use eframe::egui;
use eframe::egui::{ColorImage, DragValue, Response, ScrollArea, TextureOptions, Ui};
use dbpf::internal_file::resource_collection::texture_resource::{TextureResource, TextureResourceData};
use crate::editor::Editor;

#[derive(Default)]
pub struct TextureResourceEditorState {
    textures: Vec<Vec<Option<egui::TextureHandle>>>,
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

impl Editor for TextureResource {
    type EditorState = TextureResourceEditorState;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState {
        Self::EditorState {
            textures: self.decompress_all().into_iter().enumerate().map(|(tex_num, texture)| {
                texture.into_iter().enumerate().rev().map(|(mip_num, mip)| {
                    mip.ok().map(|decoded| {
                        context.load_texture(
                            format!("texture{tex_num}_mip{mip_num}"),
                            ColorImage::from_rgba_unmultiplied(
                                [decoded.width, decoded.height],
                                &decoded.data),
                            TextureOptions::NEAREST
                        )
                    })
                }).collect()
            }).collect()
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
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
        });
        ui.label(format!("format: {:?}", self.format));
        ui.label(format!("purpose: {}", self.purpose));
        for (texture_num, texture) in self.textures.iter().enumerate() {
            ui.separator();
            ui.label(format!("creator id: {:08X}, flag: {:X}",
                             texture.creator_id,
                             texture.format_flag));

            ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (mip_num, mip_level) in texture.entries.iter().enumerate() {
                        match mip_level {
                            TextureResourceData::Embedded(_) => {
                                let image = state.textures[texture_num][mip_num].as_ref().unwrap();

                                ui.image(image);
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
        
        res
    }
}
