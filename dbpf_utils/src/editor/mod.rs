use std::fmt::Debug;
use std::hash::Hash;
use eframe::egui;
use eframe::egui::{Grid, Response, Ui};
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::internal_file::behaviour_function::BehaviourFunction;
use dbpf::internal_file::DecodedFile;
use dbpf::internal_file::sim_outfits::SimOutfits;
use dbpf::internal_file::text_list::TextList;
use crate::editor::resource_collection::ResourceCollectionEditorState;

mod resource_collection;
mod sim_outfits;
mod cpf;
mod common;
mod text_list;
mod r#enum;
mod behaviour_function;

pub trait Editor {
    type EditorState: Default;

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {
        Self::EditorState::default()
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response;
}

#[derive(Debug, Default)]
pub enum DecodedFileEditorState {
    ResourceCollection(ResourceCollectionEditorState),
    SimOutfits(<SimOutfits as Editor>::EditorState),
    TextList(<TextList as Editor>::EditorState),
    BehaviourFunction(<BehaviourFunction as Editor>::EditorState),
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
            DecodedFile::TextList(str) => {
                DecodedFileEditorState::TextList(str.new_editor(context))
            }
            DecodedFile::BehaviourFunction(bhav) => {
                DecodedFileEditorState::BehaviourFunction(bhav.new_editor(context))
            }
            _ => DecodedFileEditorState::None,
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        match (self, state) {
            (DecodedFile::PropertySet(gzps), _) => gzps.show_editor(&mut (), ui),
            (DecodedFile::BinaryIndex(binx), _) => binx.show_editor(&mut (), ui),
            (DecodedFile::GenericCPF(cpf), _) => cpf.show_editor(&mut (), ui),
            (DecodedFile::SimOutfits(skin),
                DecodedFileEditorState::SimOutfits(state)) => {
                skin.show_editor(state, ui)
            }
            (DecodedFile::ResourceCollection(rcol),
                DecodedFileEditorState::ResourceCollection(state)) => {
                rcol.show_editor(state, ui)
            }
            (DecodedFile::TextList(str),
                DecodedFileEditorState::TextList(state)) => {
                str.show_editor(state, ui)
            },
            (DecodedFile::BehaviourFunction(bhav),
                DecodedFileEditorState::BehaviourFunction(state)) => {
                bhav.show_editor(state, ui)
            },
            _ => panic!(),
        }
    }
}

pub fn editor_supported(file_type: DBPFFileType) -> bool {
    match file_type {
        DBPFFileType::Known(
            // CPF
            KnownDBPFFileType::TrackSettings |
            KnownDBPFFileType::FloorXML |
            KnownDBPFFileType::NeighbourhoodObjectXML |
            KnownDBPFFileType::WantsXML |
            KnownDBPFFileType::MeshOverlayXML |
            KnownDBPFFileType::BinaryIndex |
            KnownDBPFFileType::FaceModifierXML |
            KnownDBPFFileType::TextureOverlayXML |
            KnownDBPFFileType::FenceXML |
            KnownDBPFFileType::SkinToneXML |
            KnownDBPFFileType::MaterialOverride |
            KnownDBPFFileType::Collection |
            KnownDBPFFileType::FaceNeutralXML |
            KnownDBPFFileType::HairToneXML |
            KnownDBPFFileType::FaceRegionXML |
            KnownDBPFFileType::FaceArchetypeXML |
            KnownDBPFFileType::SimDataXML |
            KnownDBPFFileType::RoofXML |
            KnownDBPFFileType::PetBodyOptions |
            KnownDBPFFileType::WallXML |
            KnownDBPFFileType::PropertySet |
            KnownDBPFFileType::SimDNA |
            KnownDBPFFileType::VersionInformation |
            KnownDBPFFileType::SimOutfits |

            // RCOL
            KnownDBPFFileType::TextureResource |
            KnownDBPFFileType::MaterialDefinition |

            // STR
            KnownDBPFFileType::TextList |
            KnownDBPFFileType::CatalogDescription |
            KnownDBPFFileType::PieMenuStrings |

            KnownDBPFFileType::SimanticsBehaviourFunction
        ) => true,
        _ => false,
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum VecEditorStateStorage<T: Editor>
where
    T::EditorState: Clone + Debug,
{
    Vec(Vec<T::EditorState>),
    Shared(T::EditorState),
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct VecEditorState<T: Editor>
where
    T::EditorState: Clone + Debug + Hash + Eq + PartialEq,
{
    /// number of columns (besides the delete button) that the editor for a single element will create
    columns: usize,
    storage: VecEditorStateStorage<T>,
}

impl<T: Editor> Default for VecEditorState<T>
where
    T::EditorState: Clone + Debug + Hash + Eq + PartialEq,
{
    fn default() -> Self {
        Self {
            columns: 1,
            storage: VecEditorStateStorage::Vec(vec![]),
        }
    }
}

impl<T: Editor + Default> Editor for Vec<T>
where
    T::EditorState: Clone + Debug + Hash + Eq + PartialEq,
{
    type EditorState = VecEditorState<T>;

    fn new_editor(&self, context: &egui::Context) -> Self::EditorState {
        Self::EditorState {
            columns: 1,
            storage: VecEditorStateStorage::Vec(self.iter().map(|elem| elem.new_editor(context)).collect()),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = Grid::new("generic vector editor")
            .min_col_width(0.0)
            .striped(true)
            .num_columns(state.columns + 1)
            .show(ui, |ui| {
                let (del, res): (Vec<_>, Vec<_>) = self.iter_mut()
                    .enumerate()
                    .map(|(i, elem)| {
                        let state = match &mut state.storage {
                            VecEditorStateStorage::Vec(v) => &mut v[i],
                            VecEditorStateStorage::Shared(s) => s,
                        };

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
                if let VecEditorStateStorage::Vec(v) = &mut state.storage {
                    v.retain(|_| !*it.next().unwrap());
                }

                // add new element
                let mut bres = ui.button("➕");
                if bres.clicked() {
                    let new = T::default();
                    if let VecEditorStateStorage::Vec(v) = &mut state.storage {
                        v.push(new.new_editor(ui.ctx()));
                    }
                    self.push(new);
                    bres.mark_changed();
                }

                res.into_iter().fold(bres, |r1, r2| r1 | r2)
            });
        res.response | res.inner
    }
}
