use crate::editor::{Editor, VecEditorState, VecEditorStateStorage};
use dbpf::common::LanguageCode;
use dbpf::internal_file::text_list::{TextList, TaggedString, UntaggedString, VersionedTextList, Version};
use eframe::egui::{ComboBox, Context, Response, Ui};
use crate::editor::r#enum::EnumEditorState;

impl Editor for TaggedString {
    type EditorState = <LanguageCode as Editor>::EditorState;

    fn new_editor(&self, context: &Context) -> Self::EditorState {
        LanguageCode::new_editor(&self.language_code, context)
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = self.language_code.show_editor(state, ui);
        res |= self.value.show_editor(&mut (), ui);
        res | self.description.show_editor(&mut (), ui)
    }
}

impl Editor for UntaggedString {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        self.value.show_editor(&mut (), ui)
    }
}

impl Editor for TextList {
    type EditorState = VecEditorState<TaggedString>;

    fn new_editor(&self, _context: &Context) -> Self::EditorState {
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

        let mut cur_version = match self.data {
            VersionedTextList::Tagged { version, .. } => Some(version),
            VersionedTextList::Untagged { .. } => None,
        };

        let res = res.response | res.inner |
            ui.horizontal(|ui| {
                ui.label("version tag");
                ComboBox::from_id_salt("version")
                    .selected_text(if let Some(version) = cur_version {
                        format!("{version:?}")
                    } else {
                        "None".to_string()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cur_version, None, "None");
                        ui.selectable_value(&mut cur_version, Some(Version::V0), "V0");
                        ui.selectable_value(&mut cur_version, Some(Version::V1), "V1");
                        ui.selectable_value(&mut cur_version, Some(Version::V2), "V2");
                        ui.selectable_value(&mut cur_version, Some(Version::V9), "V9");
                    }).response
            }).inner;


        match (&mut self.data, cur_version) {
            (VersionedTextList::Tagged { version, .. }, Some(new_version))
            if *version != new_version => {
                *version = new_version;
            }
            (VersionedTextList::Tagged { sets, .. }, None) => {
                self.data = VersionedTextList::Untagged {
                    sets: std::mem::take(sets).into_iter().map(|str| str.into()).collect(),
                };
            }
            (VersionedTextList::Untagged { sets }, Some(new_version)) => {
                self.data = VersionedTextList::Tagged {
                    version: new_version,
                    sets: std::mem::take(sets).into_iter().map(|str| str.into()).collect(),
                };
            }
            _ => {}
        }

        res | match &mut self.data {
            VersionedTextList::Tagged { sets, .. } => {
                sets.show_editor(state, ui)
            }
            VersionedTextList::Untagged { sets } => {
                sets.show_editor(&mut VecEditorState {
                    columns: 1,
                    storage: VecEditorStateStorage::Shared(()),
                }, ui)
            }
        }
    }
}
