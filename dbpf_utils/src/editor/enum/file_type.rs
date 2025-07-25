use crate::editor::r#enum::{EnumEditor, EnumEditorState};
use crate::editor::Editor;
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use eframe::egui::{Response, Ui};
use std::fmt::Write;

impl EnumEditor for DBPFFileType {
    type KnownEnum = KnownDBPFFileType;

    fn from_known(known_enum: &Self::KnownEnum) -> Self {
        Self::Known(*known_enum)
    }

    fn from_string(string: &String) -> Option<Self> {
        u32::from_str_radix(string.trim_start_matches("0x"), 16)
            .map(|i| DBPFFileType::from(i))
            .ok()
    }

    fn known_name(known_enum: &Self::KnownEnum) -> String {
        format!("{known_enum:?}")
    }

    fn full_name(&self) -> String {
        match self {
            DBPFFileType::Known(known) => Self::known_name(known),
            DBPFFileType::Unknown(i) => format!("{i}"),
        }
    }

    fn known_hover_string(file_type: &Self::KnownEnum) -> String {
        let mut str = String::new();
        writeln!(str, "{}", file_type.properties().name.to_string()).unwrap();
        writeln!(str, "Abbreviation: {}", file_type.properties().abbreviation).unwrap();
        if let Some(extension) = file_type.properties().extension {
            writeln!(str, "Extension: {}", extension).unwrap();
        }
        write!(str, "Id: {:08X}", *file_type as u32).unwrap();
        str
    }

    fn hover_string(&self) -> Option<String> {
        match self {
            DBPFFileType::Known(known) => {
                Some(Self::known_hover_string(known))
            }
            DBPFFileType::Unknown(_) => None,
        }
    }

    fn search_strings(file_type: &Self::KnownEnum) -> Vec<String> {
        let prop = file_type.properties();
        let mut res = vec![
            prop.name.to_string(),
            prop.abbreviation.to_string(),
            format!("{:08X}", *file_type as u32)];
        if let Some(ext) = prop.extension {
            res.push(ext.to_string());
        }
        res
    }

    fn all_known() -> impl Iterator<Item=Self::KnownEnum> {
        enum_iterator::all()
    }
}

impl Editor for DBPFFileType {
    type EditorState = EnumEditorState;

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        Self::show_enum_editor(self, state, ui)
    }
}
