use crate::editor::r#enum::{EnumEditor, EnumEditorState};
use crate::editor::Editor;
use dbpf::common::{KnownLanguageCode, LanguageCode};
use eframe::egui;
use eframe::egui::{Response, Ui};
use std::fmt::Write;
use std::str::FromStr;

impl EnumEditor for LanguageCode {
    type KnownEnum = KnownLanguageCode;

    fn from_known(known_enum: &Self::KnownEnum) -> Self {
        Self::Known(*known_enum)
    }

    fn from_int_string(string: &String) -> Option<Self> {
        u8::from_str(string).ok().map(|i| Self::from(i))
    }

    fn known_name(known_enum: &Self::KnownEnum) -> String {
        format!("{known_enum:?}")
    }

    fn full_name(&self) -> String {
        match self {
            LanguageCode::Known(known) => Self::known_name(known),
            LanguageCode::Unknown(i) => format!("{i}"),
        }
    }

    fn known_hover_string(known_enum: &Self::KnownEnum) -> String {
        let mut str = String::new();
        writeln!(str, "{:?}", known_enum).unwrap();
        write!(str, "Id: {}", *known_enum as u8).unwrap();
        str
    }

    fn hover_string(&self) -> Option<String> {
        match self {
            LanguageCode::Known(known) => {
                Some(Self::known_hover_string(known))
            }
            LanguageCode::Unknown(_) => None,
        }
    }

    fn search_strings(known_enum: &Self::KnownEnum) -> Vec<String> {
        vec![format!("{known_enum:?}"),
             format!("{}", *known_enum as u8)]
    }
}

impl Editor for LanguageCode {
    type EditorState = EnumEditorState;

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {
        Self::new_enum_editor()
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        Self::show_enum_editor(self, state, ui)
    }
}
