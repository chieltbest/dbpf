use crate::editor::r#enum::EnumEditorState;
use crate::editor::{Editor, VecEditorState, VecEditorStateStorage};
use dbpf::common::LanguageCode;
use dbpf::internal_file::text_list::{String, TextList};
use eframe::egui;
use eframe::egui::{Response, Ui};

impl Editor for String {
    type EditorState = <LanguageCode as Editor>::EditorState;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState {
        LanguageCode::new_editor(&self.language_code, context)
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = self.language_code.show_editor(state, ui);
        res |= self.value.show_editor(&mut (), ui);
        res | self.description.show_editor(&mut (), ui)
    }
}

impl Editor for TextList {
    type EditorState = VecEditorState<String>;

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {
        VecEditorState {
            columns: 3,
            storage: VecEditorStateStorage::Shared(EnumEditorState::default()),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = ui.horizontal_wrapped(|ui| {
            ui.label("file name") |
            self.file_name.name.show_editor(&mut (), ui)
        });

        res.response | res.inner | self.sets.show_editor(state, ui)
    }
}
