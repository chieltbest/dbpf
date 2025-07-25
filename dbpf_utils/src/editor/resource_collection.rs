use eframe::egui;
use eframe::egui::{DragValue, Response, ScrollArea, Ui};
use dbpf::internal_file::resource_collection::{ResourceCollection, ResourceData};
use crate::editor::Editor;
use texture_resource::TextureResourceEditorState;

mod material_definition;
mod texture_resource;

#[derive(Debug)]
#[non_exhaustive]
pub enum ResourceEditorState {
    TextureResource(TextureResourceEditorState),
    None,
}

#[derive(Debug, Default)]
pub struct ResourceCollectionEditorState {
    pub resource_editor_states: Vec<ResourceEditorState>,
}

impl Editor for ResourceCollection {
    type EditorState = ResourceCollectionEditorState;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState {
        Self::EditorState {
            resource_editor_states: self.entries.iter().map(|entry| {
                match &entry.data {
                    ResourceData::Texture(texture) => {
                        ResourceEditorState::TextureResource(texture.new_editor(context))
                    }
                    ResourceData::Material(_material) => {
                        ResourceEditorState::None
                    }
                    ResourceData::Mesh(_) => todo!(),
                }
            }).collect(),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        ScrollArea::vertical().show(ui, |ui| {
            let mut res = ui.checkbox(&mut self.version, "Has resource id");

            ui.separator();

            for link in &mut self.links {
                ui.horizontal(|ui| {
                    ui.label(link.type_id.properties()
                        .map(|prop| prop.name.to_string())
                        .unwrap_or_else(|| format!("{:08X}", link.type_id.code())));
                    res |= ui.add(DragValue::new(&mut link.group_id));
                    res |= ui.add(DragValue::new(&mut link.instance_id));
                    res |= ui.add(DragValue::new(&mut link.resource_id));
                });
            }

            for (num, entry) in self.entries.iter_mut().enumerate() {
                ui.label(entry.type_id.properties()
                    .map(|prop| prop.name.to_string())
                    .unwrap_or_else(|| format!("{:08X}", entry.type_id.code())));

                match &mut entry.data {
                    ResourceData::Texture(texture) => {
                        match &mut state.resource_editor_states[num] {
                            ResourceEditorState::TextureResource(tex_edit_state) => {
                                res |= texture.show_editor(tex_edit_state, ui);
                            }
                            _ => {
                                panic!()
                            }
                        }
                    }
                    ResourceData::Material(material) => {
                        match &mut state.resource_editor_states[num] {
                            ResourceEditorState::None => {
                                res |= material.show_editor(&mut (), ui);
                            }
                            _ => {
                                panic!()
                            }
                        }
                    }
                    ResourceData::Mesh(_) => todo!(),
                }
            }
            
            res
        }).inner
    }
}
