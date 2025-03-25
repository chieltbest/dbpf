use eframe::egui;
use eframe::egui::{DragValue, Response, Ui};
use egui_extras::Column;
use dbpf::common;
use dbpf::internal_file::common::cpf::{CPF, Data};
use crate::editor::Editor;

impl Editor for CPF {
    type EditorState = ();

    fn new_editor(&self, _context: &egui::Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        fn typed_drag_value(value: &mut Data, ui: &mut Ui) -> Response {
            let res = match value {
                Data::UInt(n) => ui.add(DragValue::new(n)),
                Data::String(s) => s.show_editor(&mut (), ui),
                Data::Float(n) => ui.add(DragValue::new(n)),
                Data::Bool(b) => {
                    let mut new = *b > 0;
                    let res = ui.checkbox(&mut new, "");
                    if res.changed() {
                        *b = new as u8;
                    }
                    res
                }
                Data::Int(n) => ui.add(DragValue::new(n)),
            };
            res.context_menu(|ui| {
                if ui.selectable_label(matches!(value, Data::UInt(_)), "UInt").clicked() {
                    *value = Data::UInt(0);
                    ui.close_menu();
                }
                if ui.selectable_label(matches!(value, Data::Int(_)), "Int").clicked() {
                    *value = Data::Int(0);
                    ui.close_menu();
                }
                if ui.selectable_label(matches!(value, Data::Float(_)), "Float").clicked() {
                    *value = Data::Float(0.0);
                    ui.close_menu();
                }
                if ui.selectable_label(matches!(value, Data::Bool(_)), "Bool").clicked() {
                    *value = Data::Bool(0);
                    ui.close_menu();
                }
                if ui.selectable_label(matches!(value, Data::String(_)), "String").clicked() {
                    *value = Data::String(common::String::default());
                    ui.close_menu();
                }
            });
            res
        }

        let button_height = ui.style().spacing.interact_size.y;

        let ires = ui.horizontal_wrapped(|ui| {
            ui.label("Version: ");
            ui.add(DragValue::new(&mut self.version))
        });
        let mut res = ires.response | ires.inner;
        ui.separator();
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto()
                .resizable(true))
            .column(Column::remainder())
            .header(button_height, |mut row| {
                row.col(|ui| { ui.label("Name"); });
                row.col(|ui| { ui.label("Value"); });
            })
            .body(|body| {
                body.rows(
                    button_height,
                    self.entries.len(),
                    |mut row| {
                        let entry = &mut self.entries[row.index()];
                        row.col(|ui| {
                            res |= entry.name.show_editor(&mut (), ui);
                        });
                        row.col(|ui| {
                            res |= typed_drag_value(&mut entry.data, ui);
                        });
                    });
            });
        
        res
    }
}
