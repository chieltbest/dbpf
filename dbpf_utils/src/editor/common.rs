use std::sync::Arc;
use crate::editor::Editor;
use binrw::NullWideString;
use dbpf::common::{BigString, ByteString, NullString, PascalString};
use eframe::{egui, glow};
use eframe::egui::{Response, TextEdit, Ui, Vec2};

trait StringEditor: TryInto<String> + From<String> + Clone {}

impl<T: ?Sized + StringEditor> Editor for T {
    type EditorState = f32;

    fn new_editor(&self, _context: &egui::Context, _gl_context: &Option<Arc<glow::Context>>) -> Self::EditorState {
        300.0
    }
    
    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let string_res = self.clone().try_into();
        match string_res {
            Ok(mut str) => {
                let text_edit = TextEdit::singleline(&mut str)
                    .min_size(Vec2::new(*state, 0.0));
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

impl StringEditor for ByteString {}

impl StringEditor for PascalString<u32> {}
impl StringEditor for PascalString<u8> {}

impl StringEditor for BigString {}

impl StringEditor for NullString {}
impl StringEditor for NullWideString {}
