use std::default::Default;
use eframe::egui;
use eframe::egui::{DragValue, Response, Ui};
use dbpf::filetypes::DBPFFileType;
use dbpf::IndexMinorVersion;
use dbpf::internal_file::sim_outfits::{Entry, SimOutfits};
use crate::editor::{Editor, VecEditorState, VecEditorStateStorage};

impl Editor for Entry {
    type EditorState = <DBPFFileType as Editor>::EditorState;

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {
        Default::default()
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = self.type_id.show_editor(state, ui);
        res |= ui.add(DragValue::new(&mut self.group_id)
            .hexadecimal(8, false, true));
        res |= ui.add(DragValue::new(&mut self.instance_id.id)
            .hexadecimal(8, false, true));
        res
    }
}

impl Editor for SimOutfits {
    type EditorState = VecEditorState<Entry>;

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {
        VecEditorState {
            columns: 3,
            storage: VecEditorStateStorage::Shared(Default::default()),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let ires = ui.horizontal_wrapped(|ui| {
            ui.label("Index version");
            let mut res = ui.selectable_value(&mut self.version, IndexMinorVersion::V1, "V1");
            res |= ui.selectable_value(&mut self.version, IndexMinorVersion::V2, "V2");
            res
        });
        let mut res = ires.response | ires.inner;

        ui.separator();

        res |= self.entries.show_editor(state, ui);
        
        res
    }
}
