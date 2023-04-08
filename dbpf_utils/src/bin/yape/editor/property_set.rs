use eframe::egui::{DragValue, Ui};
use egui_extras::Column;
use dbpf::common;
use dbpf::internal_file::cpf::Data;
use dbpf::internal_file::property_set::PropertySet;
use crate::editor::{Editor, string_editor};

impl Editor for PropertySet {
    fn show_editor(&mut self, ui: &mut Ui) {
        fn typed_drag_value(value: &mut Data, ui: &mut Ui) {
            match value {
                Data::UInt(n) => ui.add(DragValue::new(n)),
                Data::String(s) => string_editor(s, ui),
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
            }.context_menu(|ui| {
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
        }

        match self {
            PropertySet::CPF(cpf) => {
                let button_height = ui.style().spacing.interact_size.y;

                ui.horizontal_wrapped(|ui| {
                    ui.label("Version: ");
                    ui.add(DragValue::new(&mut cpf.version));
                });
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
                            cpf.entries.len(),
                            |i, mut row| {
                                let entry = &mut cpf.entries[i];
                                row.col(|ui| {
                                    string_editor(&mut entry.name, ui);
                                });
                                row.col(|ui| {
                                    typed_drag_value(&mut entry.data, ui);
                                });
                            });
                    });
            }
        }
    }
}
