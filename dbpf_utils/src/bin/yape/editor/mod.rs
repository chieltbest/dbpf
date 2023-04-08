use eframe::egui::{Response, TextEdit, Ui};
use dbpf::common;
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::internal_file::DecodedFile;

mod property_set;

pub trait Editor {
    fn show_editor(&mut self, ui: &mut Ui);
}

impl Editor for DecodedFile {
    fn show_editor(&mut self, ui: &mut Ui) {
        match self {
            DecodedFile::PropertySet(prop) => prop.show_editor(ui),
            _ => {}
        }
    }
}

pub(crate) fn editor_supported(file_type: DBPFFileType) -> bool {
    match file_type {
        DBPFFileType::Known(KnownDBPFFileType::PropertySet) => true,
        _ => false,
    }
}

fn string_editor(string: &mut common::String, ui: &mut Ui) -> Response {
    let mut res = string.clone().into_string();
    match res {
        Ok(ref mut str) => {
            ui.text_edit_singleline(str)
        }
        Err(_) => {
            ui.add_enabled(
                false,
                TextEdit::singleline(
                    &mut String::from_utf8_lossy(string.data.as_slice()).to_string()))
        }
    }
}
