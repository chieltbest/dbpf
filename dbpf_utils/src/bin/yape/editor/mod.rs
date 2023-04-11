use eframe::egui::{Response, TextEdit, Ui};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::internal_file::DecodedFile;
use crate::editor::resource_collection::ResourceCollectionEditorState;

mod property_set;
mod texture_resource;
mod resource_collection;

pub trait Editor {
    type EditorState;

    fn new_editor(&self) -> Self::EditorState;

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui);
}

pub enum DecodedFileEditorState {
    ResourceCollection(ResourceCollectionEditorState),
    None,
}

impl Editor for DecodedFile {
    type EditorState = DecodedFileEditorState;

    fn new_editor(&self) -> Self::EditorState {
        match self {
            DecodedFile::TextureResource(rcol) => {
                DecodedFileEditorState::ResourceCollection(rcol.new_editor())
            }
            _ => DecodedFileEditorState::None,
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        match self {
            DecodedFile::PropertySet(prop) => prop.show_editor(&mut (), ui),
            DecodedFile::TextureResource(rcol) => {
                if let DecodedFileEditorState::ResourceCollection(rcol_state) = state {
                    rcol.show_editor(rcol_state, ui);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn editor_supported(file_type: DBPFFileType) -> bool {
    match file_type {
        DBPFFileType::Known(KnownDBPFFileType::PropertySet) |
        DBPFFileType::Known(KnownDBPFFileType::TextureResource) => true,
        _ => false,
    }
}

fn string_editor<T: TryInto<String> + From<String> + Clone>(string: &mut T, ui: &mut Ui) -> Response {
    let string_res = string.clone().try_into();
    match string_res {
        Ok(mut str) => {
            let text_edit = TextEdit::singleline(&mut str).desired_width(f32::INFINITY);
            let res = text_edit.show(ui).response;
            if res.changed() {
                *string = str.into();
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
