use crate::editor::Editor;
use dbpf::internal_file::cpf::binary_index::BinaryIndex;
use eframe::egui;
use eframe::egui::{DragValue, Grid, Response, Ui};
use eframe::emath::Numeric;

impl Editor for BinaryIndex {
    type EditorState = ();

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let ires = Grid::new("BinaryIndex edit grid")
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

                let mut res = drag!(iconidx);
                res |= drag!(stringindex);
                res |= drag!(binidx);
                res |= drag!(objectidx);
                res |= string!(creatorid);
                res |= drag!(sortindex);
                res |= drag!(stringindex);

                res
            });

        ires.response | ires.inner
    }
}
