use eframe::egui;
use eframe::egui::{Grid, Response, Ui};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::internal_file::DecodedFile;
use crate::editor::resource_collection::ResourceCollectionEditorState;
use crate::editor::sim_outfits::SimOutfitsEditorState;

mod property_set;
mod resource_collection;
mod sim_outfits;
mod file_type;
mod cpf;
mod common;
mod text_list;
mod binary_index;

pub trait Editor {
    type EditorState;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState;

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response;
}

#[derive(Debug, Default)]
pub enum DecodedFileEditorState {
    ResourceCollection(ResourceCollectionEditorState),
    SimOutfits(SimOutfitsEditorState),
    #[default]
    None,
}

impl Editor for DecodedFile {
    type EditorState = DecodedFileEditorState;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState {
        match self {
            DecodedFile::SimOutfits(skin) => {
                DecodedFileEditorState::SimOutfits(skin.new_editor(context))
            }
            DecodedFile::ResourceCollection(rcol) => {
                DecodedFileEditorState::ResourceCollection(rcol.new_editor(context))
            }
            _ => DecodedFileEditorState::None,
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        match (self, state) {
            (DecodedFile::PropertySet(gzps), _) => gzps.show_editor(&mut (), ui),
            (DecodedFile::BinaryIndex(binx), _) => binx.show_editor(&mut (), ui),
            (DecodedFile::SimOutfits(skin),
                DecodedFileEditorState::SimOutfits(state)) => {
                skin.show_editor(state, ui)
            }
            (DecodedFile::ResourceCollection(rcol),
                DecodedFileEditorState::ResourceCollection(state)) => {
                rcol.show_editor(state, ui)
            }
            (DecodedFile::TextList(str), _) => str.show_editor(&mut (), ui),
            _ => panic!(),
        }
    }
}

pub fn editor_supported(file_type: DBPFFileType) -> bool {
    match file_type {
        DBPFFileType::Known(
            KnownDBPFFileType::PropertySet |
            KnownDBPFFileType::BinaryIndex |
            KnownDBPFFileType::SimOutfits |
            KnownDBPFFileType::TextureResource |
            KnownDBPFFileType::MaterialDefinition |
            KnownDBPFFileType::TextList |
            KnownDBPFFileType::CatalogDescription |
            KnownDBPFFileType::CatalogString |
            KnownDBPFFileType::PieMenuStrings
        ) => true,
        _ => false,
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct VecEditorState<T: Editor> {
    columns: usize,
    elem_states: Vec<T::EditorState>,
}

impl<T: Editor> Default for VecEditorState<T> {
    fn default() -> Self {
        Self {
            columns: 1,
            elem_states: vec![],
        }
    }
}

impl<T: Editor + Default> Editor for Vec<T> {
    type EditorState = VecEditorState<T>;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState {
        Self::EditorState {
            columns: 1,
            elem_states: self.iter().map(|elem| elem.new_editor(context)).collect(),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = Grid::new("generic vector editor")
            .min_col_width(0.0)
            .striped(true)
            .num_columns(state.columns + 1)
            .show(ui, |ui| {
                let (del, res): (Vec<_>, Vec<_>) = self.iter_mut()
                    .zip(state.elem_states.iter_mut())
                    .enumerate()
                    .map(|(i, (elem, state))| {
                        let del = ui.button("🗑").clicked();

                        let ires = ui.push_id(i, |ui| {
                            elem.show_editor(state, ui)
                        });
                        let mut res = ires.response | ires.inner;
                        if del {
                            res.mark_changed();
                        }

                        ui.end_row();

                        (del, res)
                    }).collect();

                let mut it = del.iter();
                self.retain(|_| !*it.next().unwrap());
                it = del.iter();
                state.elem_states.retain(|_| !*it.next().unwrap());

                // add new element
                let mut bres = ui.button("➕");
                if bres.clicked() {
                    let new = T::default();
                    state.elem_states.push(new.new_editor(ui.ctx()));
                    self.push(new);
                    bres.mark_changed();
                }

                res.into_iter().fold(bres, |r1, r2| r1 | r2)
            });
        res.response | res.inner
    }
}
