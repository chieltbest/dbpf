use eframe::egui;
use eframe::egui::{Align, DragValue, Grid, Response, Ui};
use eframe::emath::Numeric;
use dbpf::internal_file::property_set::{Override, PropertySet};
use crate::editor::{Editor, VecEditorState};

impl Editor for Override {
    type EditorState = ();

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = ui.add(DragValue::new(&mut self.shape)).on_hover_text("shape");
        res |= ui.add(DragValue::new(&mut self.resourcekeyidx)).on_hover_text("resource key index");
        res | self.subset.show_editor(&mut (), ui)
    }
}

impl Editor for PropertySet {
    type EditorState = ();

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let ires = Grid::new("PropertySet edit grid")
            .num_columns(2)
            .show(ui, |ui| {
                fn drag_fn<T: Numeric>(name: &str, value: &mut T, ui: &mut Ui) -> Response {
                    ui.label(name);
                    let res = ui.add(DragValue::new(value).hexadecimal(1, false, false));
                    res
                }

                macro_rules! drag {
                    ($name:ident) => {
                        {
                            let res = drag_fn(stringify!($name), &mut self.$name, ui);
                            ui.end_row();
                            res
                        }
                    };
                }

                fn drag_checkbox_fn<const N: usize>(name: &str, value: &mut u32, bit_names: [&str; N], ui: &mut Ui) -> Response {
                    ui.label(name);
                    let res = ui.with_layout(
                        egui::Layout::left_to_right(Align::TOP).with_main_wrap(true),
                        |ui| {
                            let res = ui.add(DragValue::new(value).hexadecimal(1, false, false));

                            bit_names.iter().enumerate().fold(res, |res, (i, c_name)| {
                                let mask = 1 << i;
                                let o = (*value & mask) > 0;
                                let mut c = o;
                                let res = res | ui.checkbox(&mut c, *c_name);
                                if c != o {
                                    *value = (*value & !mask) | (
                                        if c {
                                            1
                                        } else {
                                            0
                                        } << i
                                    );
                                }
                                res
                            })
                        });
                    ui.end_row();
                    res.response | res.inner
                }

                macro_rules! drag_checkbox {
                    ($name:ident, $($c_name:expr),*) => {
                        drag_checkbox_fn(stringify!($name), &mut self.$name, [$($c_name),*], ui)
                    };
                }
                macro_rules! string {
                    ($name:ident) => {
                        {
                            ui.label(stringify!($name));
                            let res = self.$name.show_editor(&mut (), ui);
                            ui.end_row();
                            res
                        }
                    };
                }

                let mut res = drag!(version);
                res |= drag!(product);
                res |= drag_checkbox!(age, "toddler", "child", "teen", "adult", "elder", "baby", "young adult");
                res |= drag_checkbox!(gender, "female", "male");
                res |= drag!(species);
                res |= drag!(parts);
                res |= drag_checkbox!(outfit, "hair", "face", "top", "body", "bottom", "accessory", "long tail",
                    "upright ears", "short tail", "floppy ears", "long brush tail", "short brush tail",
                    "spitz tail", "brush spitz tail");
                res |= drag!(flags);
                res |= string!(name);
                res |= string!(creator);
                res |= string!(family);
                res |= drag!(genetic);
                res |= drag!(priority);

                // type is a builtin keyword, so use a different name
                ui.label("type");
                res |= self.type_.show_editor(&mut (), ui);
                ui.end_row();

                res |= string!(skintone);
                res |= string!(hairtone);
                res |= drag_checkbox!(category, "casual 1", "casual 2", "casual 3", "swimwear",
                    "sleepwear", "formal", "underwear", "skintone", "pregnant", "activewear", "try on", "naked overlay",
                    "outerwear");
                res |= drag!(shoe);
                res |= drag!(fitness);
                res |= drag!(resourcekeyidx);
                res |= drag!(shapekeyidx);

                res
            });

        ires.response | ires.inner | self.overrides.show_editor(&mut VecEditorState {
            columns: 3,
            elem_states: vec![(); self.overrides.len()],
        }, ui)
    }
}
