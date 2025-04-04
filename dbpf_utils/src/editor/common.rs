use binrw::{NullString, NullWideString};
use eframe::egui;
use eframe::egui::{Response, TextEdit, Ui, Vec2};
use dbpf::common;
use dbpf::common::BigString;
use crate::editor::Editor;

trait StringEditor {}

impl<T: StringEditor + TryInto<String> + From<String> + Clone> Editor for T {
    type EditorState = ();

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {}
    
    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let string_res = self.clone().try_into();
        match string_res {
            Ok(mut str) => {
                let text_edit = TextEdit::singleline(&mut str)
                    .min_size(Vec2::new(300.0, 0.0));
                let res = text_edit.show(ui).response;
                if res.changed() {
                    *self = str.into();
                }
                res
            }
            Err(_) => {
                // ui.add_enabled(
                //     false,
                //     TextEdit::singleline(
                //         &mut String::from_utf8_lossy(string.data.as_slice()).to_string()))
                ui.label("non-utf8 string")
            }
        }
    }
}

impl StringEditor for common::String {}

impl StringEditor for BigString {}

impl StringEditor for NullString {}

impl StringEditor for NullWideString {}
