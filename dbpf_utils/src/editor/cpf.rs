use eframe::egui;
use eframe::egui::{DragValue, Response, Ui};
use egui_extras::Column;
use dbpf::common;
use dbpf::internal_file::cpf::{CPF, Data, CPFVersion, DataType};
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
                Data::Bool(b) => ui.checkbox(b, ""),
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
                    *value = Data::Bool(false);
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
            let mut res = egui::ComboBox::new("cpf_version", "Version ")
                .selected_text(if matches!(self.version, CPFVersion::XML(_, _)) {
                    "XML"
                } else {
                    "CPF"
                })
                .show_ui(ui, |ui| {
                    if ui.button("XML").clicked() {
                        self.version = CPFVersion::XML(DataType::String, None); // TODO version selector
                    }
                    if ui.button("CPF").clicked() {
                        self.version = CPFVersion::CPF(2);
                    }
                }).response;
            if let CPFVersion::CPF(version) = &mut self.version {
                res |= ui.add(DragValue::new(version))
            }
            res
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
