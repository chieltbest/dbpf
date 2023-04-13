use eframe::egui::{DragValue, Ui};
use dbpf::IndexMinorVersion;
use dbpf::internal_file::sim_outfits::{Entry, SimOutfits};
use crate::editor::Editor;

impl Editor for Entry {
    type EditorState = ();

    fn new_editor(&self) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) {
        ui.label(self.type_id.full_name());
        ui.add(DragValue::new(&mut self.group_id)
            .hexadecimal(8, false, true));
        ui.add(DragValue::new(&mut self.instance_id.id)
            .hexadecimal(8, false, true));
    }
}

impl Editor for SimOutfits {
    type EditorState = ();

    fn new_editor(&self) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Index version");
            ui.selectable_value(&mut self.version, IndexMinorVersion::V1, "V1");
            ui.selectable_value(&mut self.version, IndexMinorVersion::V2, "V2");
        });

        ui.separator();

        self.entries.show_editor(&mut self.entries.new_editor(), ui);
    }
}
