use eframe::egui::{Grid, Response, TextEdit, Ui};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::internal_file::DecodedFile;
use crate::editor::resource_collection::ResourceCollectionEditorState;

mod property_set;
mod resource_collection;
mod sim_outfits;
mod texture_resource;

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
            DecodedFile::PropertySet(gzps) => gzps.show_editor(&mut (), ui),
            DecodedFile::SimOutfits(skin) => skin.show_editor(&mut (), ui),
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
        DBPFFileType::Known(KnownDBPFFileType::SimOutfits) |
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

impl<T: Editor + Default> Editor for Vec<T> {
    /// number of columns and vec of T editor states
    type EditorState = (usize, Vec<T::EditorState>);

    fn new_editor(&self) -> Self::EditorState {
        (2, self.iter().map(|elem| elem.new_editor()).collect())
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        Grid::new("generic vector editor")
            .min_col_width(0.0)
            .striped(true)
            .num_columns(state.0 + 1)
            .show(ui, |ui| {
                let del: Vec<_> = self.iter_mut().zip(state.1.iter_mut()).map(|(elem, state)| {
                    let del = ui.button("🗑").clicked();

                    elem.show_editor(state, ui);

                    ui.end_row();

                    del
                }).collect();

                let mut it = del.iter();
                self.retain(|_| !*it.next().unwrap());
                it = del.iter();
                state.1.retain(|_| !*it.next().unwrap());

                // add new element
                if ui.button("🗋").clicked() {
                    let new = T::default();
                    state.1.push(new.new_editor());
                    self.push(new);
                }
            });
    }
}
