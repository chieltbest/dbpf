use eframe::egui;
use eframe::egui::{Response, Ui};
use dbpf::common::LanguageCode;
use dbpf::internal_file::text_list::{String, TextList};
use crate::editor::{Editor, VecEditorState};

impl Editor for String {
    type EditorState = ();

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = egui::ComboBox::new("language code", "")
            .selected_text(format!("{:?}", self.language_code))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.language_code, LanguageCode::English, "English")
            });
        let mut res = if let Some(inner) = res.inner {
            res.response | inner
        } else {
            res.response
        };
        res |= self.value.show_editor(&mut (), ui);
        res | self.description.show_editor(&mut (), ui)
    }
}

impl Editor for TextList {
    type EditorState = ();

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = ui.horizontal_wrapped(|ui| {
            ui.label("file name") |
            self.file_name.name.show_editor(&mut (), ui)
        });

        res.response | res.inner | self.sets.show_editor(&mut VecEditorState {
            columns: 3,
            elem_states: vec![(); self.sets.len()],
        }, ui)
    }
}
