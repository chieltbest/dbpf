pub mod file_type;
pub mod language_code;

use eframe::egui::text::{CCursor, CCursorRange};
use eframe::egui::{Align, Key, Response, ScrollArea, TextEdit, Ui};
use fuzzy_matcher::FuzzyMatcher;

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct EnumEditorState {
    search_string: String,
    focus_self: bool,
}

pub trait EnumEditor {
    type KnownEnum;

    fn from_known(known_enum: &Self::KnownEnum) -> Self;

    fn from_int_string(string: &String) -> Option<Self>
    where
        Self: Sized;

    fn known_name(known_enum: &Self::KnownEnum) -> String;

    fn full_name(&self) -> String;

    fn known_hover_string(known_enum: &Self::KnownEnum) -> String;

    fn hover_string(&self) -> Option<String>;

    fn search_strings(known_enum: &Self::KnownEnum) -> Vec<String>;

    fn all_known() -> impl Iterator<Item=Self::KnownEnum>;

    fn show_enum_editor(&mut self, state: &mut EnumEditorState, ui: &mut Ui) -> Response
    where
        Self: PartialEq,
        Self: Sized,
    {
        let inner_res = ui.menu_button(self.full_name(), |ui| {
            let mut text_edit_response = TextEdit::singleline(&mut state.search_string).show(ui);

            if state.focus_self {
                state.focus_self = false;
                text_edit_response.state.cursor.set_char_range(Some(CCursorRange {
                    secondary: CCursor::new(0),
                    primary: CCursor::new(state.search_string.len()),
                }));
                text_edit_response.state.clone().store(ui.ctx(), text_edit_response.response.id);
                text_edit_response.response.request_focus();
            }

            let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
            let mut scored_types = Self::all_known()
                .filter_map(|t| {
                    Self::search_strings(&t)
                        .into_iter()
                        .map(|str| {
                            matcher.fuzzy_match(str.as_str(), state.search_string.as_str())
                        })
                        .max().unwrap()
                        .map(|score| (t, score))
                }).collect::<Vec<_>>();
            scored_types.sort_unstable_by_key(|(_, score)| -score);

            ScrollArea::vertical()
                .min_scrolled_height(200.0)
                .max_height(200.0)
                .show(ui, |ui| {
                    if text_edit_response.response.changed() {
                        ui.scroll_to_cursor(Some(Align::TOP));
                    }
                    scored_types.iter().for_each(|(t, _)| {
                        if ui.selectable_label(
                            *self == Self::from_known(t),
                            Self::known_name(t))
                            .on_hover_text(Self::known_hover_string(t))
                            .clicked() {
                            *self = Self::from_known(t);
                            text_edit_response.response.mark_changed();
                            ui.close_menu();
                        }
                    });
                });

            if text_edit_response.response.lost_focus() &&
                ui.input(|i| i.key_pressed(Key::Enter)) {
                if let Some((t, _score)) = scored_types.first() {
                    *self = Self::from_known(t);
                } else if let Some(new) = Self::from_int_string(&state.search_string) {
                    *self = new;
                }
                ui.close_menu();
                text_edit_response.response.mark_changed();
            }
            text_edit_response
        });
        if inner_res.response.clicked() {
            state.focus_self = true;
        }

        if let Some(str) = self.hover_string() {
            inner_res.response.clone().on_hover_text(str);
        }

        if let Some(inner) = inner_res.inner {
            inner_res.response | inner.response
        } else {
            inner_res.response
        }
    }
}
