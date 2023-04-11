use eframe::egui::{DragValue, ScrollArea, Ui};
use dbpf::internal_file::resource_collection::{ResourceCollection, ResourceData};
use crate::editor::Editor;
use crate::editor::texture_resource::TextureResourceEditorState;

pub enum ResourceEditorState {
    TextureResource(TextureResourceEditorState),
}

#[derive(Default)]
pub struct ResourceCollectionEditorState {
    pub resource_editor_states: Vec<ResourceEditorState>,
}

impl Editor for ResourceCollection {
    type EditorState = ResourceCollectionEditorState;

    fn new_editor(&self) -> Self::EditorState {
        Self::EditorState {
            resource_editor_states: self.entries.iter().map(|entry| {
                match &entry.data {
                    ResourceData::Texture(texture) => {
                        ResourceEditorState::TextureResource(texture.new_editor())
                    }
                }
            }).collect(),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.checkbox(&mut self.version, "Has resource id");

            ui.separator();

            for link in &mut self.links {
                ui.horizontal(|ui| {
                    ui.label(link.type_id.properties()
                        .map(|prop| prop.name.to_string())
                        .unwrap_or_else(|| format!("{:08X}", link.type_id.code())));
                    ui.add(DragValue::new(&mut link.group_id));
                    ui.add(DragValue::new(&mut link.instance_id));
                    ui.add(DragValue::new(&mut link.resource_id));
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
                                texture.show_editor(tex_edit_state, ui);
                            }
                        }
                    }
                }
            }
        });
    }
}
