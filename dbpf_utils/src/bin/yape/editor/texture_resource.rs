use eframe::egui::{Align, ColorImage, Layout, Ui, Vec2};
use dbpf::internal_file::texture_resource::{TextureResource, TextureResourceData};
use crate::editor::Editor;

#[derive(Default)]
pub struct TextureResourceEditorState {
    textures: Vec<Vec<Option<egui_extras::RetainedImage>>>,
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
                                    &embedded_mip.data)))
                        }
                        TextureResourceData::LIFOFile { .. } => None,
                    }
                }).collect()
            }).collect()
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        ui.label(format!("{}: {}x{}px, {} mip levels",
                         String::from_utf8_lossy(&self.file_name.name.data),
                         self.width,
                         self.height,
                         self.mip_levels));
        ui.label(format!("format: {:?}", self.format));
        ui.label(format!("purpose: {}", self.purpose));
        for (texture_num, texture) in self.textures.iter().enumerate() {
            ui.separator();
            ui.label(format!("creator id: {:08X}, flag: {:X}",
                             texture.creator_id,
                             texture.format_flag));
            for (mip_num, mip_level) in texture.entries.iter().enumerate() {
                match mip_level {
                    TextureResourceData::Embedded(_) => {
                        let texture = state.textures[texture_num][mip_num].as_ref().unwrap();
                        let available_width = ui.available_width();
                        ui.allocate_ui_with_layout(
                            texture.size_vec2(),
                            Layout::top_down_justified(Align::LEFT),
                        |ui| {
                            texture.show_max_size(ui, Vec2::new(available_width, f32::INFINITY));
                        });
                    }
                    TextureResourceData::LIFOFile { file_name } => {
                        ui.label(format!("file: {}",
                                         String::from_utf8_lossy(&file_name.data)));
                    }
                }
            }
        }
    }
}
