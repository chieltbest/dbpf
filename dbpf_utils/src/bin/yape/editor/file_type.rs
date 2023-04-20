use std::fmt::Write;
use eframe::egui::{Key, ScrollArea, TextEdit, Ui};
use eframe::egui::text::{CCursor, CCursorRange};
use fuzzy_matcher::FuzzyMatcher;
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use crate::editor::Editor;

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct DBPFFileTypeEditorState {
    search_string: String,
}

fn known_dbpf_file_type_hover_string(file_type: KnownDBPFFileType) -> String {
    let mut str = String::new();
    writeln!(str, "{}", file_type.properties().name.to_string()).unwrap();
    writeln!(str, "Abbreviation: {}", file_type.properties().abbreviation).unwrap();
    if let Some(extension) = file_type.properties().extension {
        writeln!(str, "Extension: {}", extension).unwrap();
    }
    write!(str, "Id: {:08X}", file_type as u32).unwrap();
    str
}

fn dbpf_file_type_hover_string(file_type: DBPFFileType) -> Option<String> {
    match file_type {
        DBPFFileType::Known(known) => {
            Some(known_dbpf_file_type_hover_string(known))
        }
        DBPFFileType::Unknown(_) => None,
    }
}

fn dbpf_file_type_search_strings(file_type: KnownDBPFFileType) -> Vec<String> {
    let prop = file_type.properties();
    let mut res = vec![
        prop.name.to_string(),
        prop.abbreviation.to_string(),
        format!("{:08X}", file_type as u32)];
    if let Some(ext) = prop.extension {
        res.push(ext.to_string());
    }
    res
}

impl Editor for DBPFFileType {
    type EditorState = DBPFFileTypeEditorState;

    fn new_editor(&self) -> Self::EditorState {
        Self::EditorState {
            search_string: "".to_string(),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        let mut text_edit_response = None;
        let inner_res = ui.menu_button(self.full_name(), |ui| {
            text_edit_response = Some(TextEdit::singleline(&mut state.search_string).show(ui));

            let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
            let mut scored_types = enum_iterator::all::<KnownDBPFFileType>()
                .filter_map(|t| {
                    dbpf_file_type_search_strings(t)
                        .into_iter()
                        .map(|str| {
                            matcher.fuzzy_match(str.as_str(), state.search_string.as_str())
                        })
                        .max().unwrap()
                        .map(|score| (t, score))
                }).collect::<Vec<_>>();
            scored_types.sort_unstable_by_key(|(_, score)| -score);

            ScrollArea::vertical().show(ui, |ui| {
                scored_types.iter().for_each(|(t, _)| {
                    if ui.selectable_label(
                        *self == DBPFFileType::Known(*t),
                        t.properties().name)
                        .on_hover_text(known_dbpf_file_type_hover_string(*t))
                        .clicked() {
                        *self = DBPFFileType::Known(*t);
                        ui.close_menu();
                    }
                });
            });

            if text_edit_response.as_ref().unwrap().response.lost_focus() &&
                ui.input(|i| i.key_pressed(Key::Enter)) {
                if let Some((t, _score)) = scored_types.first() {
                    *self = DBPFFileType::Known(*t);
                } else {
                    let stripped = state.search_string.trim_start_matches("0x");
                    if let Ok(i) = u32::from_str_radix(stripped, 16) {
                        *self = DBPFFileType::from(i);
                    }
                }
                ui.close_menu();
            }
        });
        if inner_res.response.clicked() {
            if let Some(mut text_res) = text_edit_response {
                text_res.state.set_ccursor_range(Some(CCursorRange {
                    secondary: CCursor::new(0),
                    primary: CCursor::new(state.search_string.len()),
                }));
                text_res.state.store(ui.ctx(), text_res.response.id);
                text_res.response.request_focus();
            }
        }

        if let Some(str) = dbpf_file_type_hover_string(*self) {
            inner_res.response.on_hover_text(str);
        }
    }
}
