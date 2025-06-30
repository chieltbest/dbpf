use std::sync::Arc;
use eframe::egui::{Context, Response, Ui};
use eframe::glow;
use dbpf::common::BigString;
use dbpf::internal_file::resource_collection::material_definition::{MaterialDefinition, Property};
use crate::editor::{Editor, VecEditorState, VecEditorStateStorage};

impl Editor for Property {
    type EditorState = ();

    fn new_editor(&self, _context: &Context, _gl: &Option<Arc<glow::Context>>) -> Self::EditorState {}

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        self.name.show_editor(&mut 300.0, ui)
            | self.value.show_editor(&mut 300.0, ui)
    }
}


impl Editor for MaterialDefinition {
    type EditorState = ();

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = self.file_name.name.show_editor(&mut 500.0, ui);
        ui.horizontal(|ui| {
            ui.label("Material Type");
            res |= self.material_type.show_editor(&mut 500.0, ui);
        });
        ui.horizontal(|ui| {
            ui.label("Material Description");
            res |= self.material_description.show_editor(&mut 500.0, ui);
        });

        // ui.just

        res |= ui.label("Properties");
        res |= self.properties.show_editor(&mut VecEditorState {
                    columns: 3,
                    storage: VecEditorStateStorage::Shared(()),
                }, ui);
        
        ui.push_id("Names", |ui| {
            ui.label("Names");
            res |= self.names.show_editor(&mut VecEditorState {
                            columns: 1,
                            storage: VecEditorStateStorage::<BigString>::Shared(500.0),
                        }, ui);
        });

        res
    }
}
