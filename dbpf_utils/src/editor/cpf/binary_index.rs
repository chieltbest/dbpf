use crate::editor::cpf::{drag_fn, reference_edit_fn};
use crate::editor::Editor;
use dbpf::internal_file::cpf::binary_index::BinaryIndex;
use eframe::egui::{Grid, Response, Ui};

impl Editor for BinaryIndex {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let ires = Grid::new("BinaryIndex edit grid")
            .num_columns(2)
            .show(ui, |ui| {
                macro_rules! drag {
                    ($name:ident) => {
                        drag_fn(stringify!($name), $name, ui)
                    };
                }
                macro_rules! reference {
                    ($name:ident) => {
                        reference_edit_fn(stringify!($name), $name, ui)
                    };
                }
                macro_rules! string {
                    ($name:ident) => {
                        {
                            ui.label(stringify!($name));
                            let res = $name.show_editor(&mut (), ui);
                            ui.end_row();
                            res
                        }
                    };
                }

                let BinaryIndex {
                    icon,
                    stringset,
                    bin,
                    object,
                    creatorid,
                    sortindex,
                    stringindex
                } = self;

                let mut res = reference!(icon);
                res |= reference!(stringset);
                res |= reference!(bin);
                res |= reference!(object);
                res |= string!(creatorid);
                res |= drag!(sortindex);
                res |= drag!(stringindex);

                res
            });

        ires.response | ires.inner
    }
}
