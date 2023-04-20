use std::fmt::{Debug, Formatter};
use eframe::egui::{ColorImage, DragValue, ScrollArea, TextureOptions, Ui};
use dbpf::internal_file::resource_collection::texture_resource::{TextureResource, TextureResourceData};
use crate::editor::{Editor, string_editor};

#[derive(Default)]
pub struct TextureResourceEditorState {
    textures: Vec<Vec<Option<egui_extras::RetainedImage>>>,
}

impl Debug for TextureResourceEditorState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.textures.iter().map(|texture| {
                texture.iter().map(|mip| {
                    mip.as_ref().map(|img| {
                        format!("{}: {:?}", img.debug_name(), img.size_vec2())
                    })
                }).collect::<Vec<_>>()
            }))
            .finish()
    }
}

impl Editor for TextureResource {
    type EditorState = TextureResourceEditorState;

    fn new_editor(&self) -> Self::EditorState {
        Self::EditorState {
            textures: self.textures.iter().enumerate().map(|(texture_num, texture)| {
                texture.entries.iter().enumerate().map(|(mip_num, mip)| {
                    match mip {
                        TextureResourceData::Embedded(embedded_mip) => {
                            Some(egui_extras::RetainedImage::from_color_image(
                                format!("texture{texture_num}_mip{mip_num}"),
                                ColorImage::from_rgba_unmultiplied(
                                    [embedded_mip.width as usize, embedded_mip.height as usize],
                                    &embedded_mip.data))
                                .with_options(TextureOptions::NEAREST))
                        }
                        TextureResourceData::LIFOFile { .. } => None,
                    }
                }).collect()
            }).collect()
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        string_editor(&mut self.file_name.name, ui);
        ui.horizontal_wrapped(|ui| {
            ui.add(DragValue::new(&mut self.width));
            ui.label("width");
            ui.add(DragValue::new(&mut self.height));
            ui.label("height");
        });
        ui.horizontal(|ui| {
            ui.add(DragValue::new(&mut self.mip_levels));
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
                                image.show(ui);
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
    }
}
