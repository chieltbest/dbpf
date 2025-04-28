use eframe::egui::{ComboBox, Context, DragValue, Response, Ui};
use egui_extras::Column;
use dbpf::common;
use dbpf::internal_file::cpf::{CPF, Data, CPFVersion, DataType, Item};
use crate::editor::{Editor, VecEditorState};

impl Editor for Item {
    type EditorState = ();

    fn new_editor(&self, _context: &Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = ComboBox::new("data_type", "")
            .selected_text(format!("{:?}", self.data.get_type()))
            .show_ui(ui, |ui| {
                if ui.button("UInt").clicked() { self.data = Data::UInt(0); }
                if ui.button("Int").clicked() { self.data = Data::Int(0); }
                if ui.button("Float").clicked() { self.data = Data::Float(0.0); }
                if ui.button("Bool").clicked() { self.data = Data::Bool(false); }
                if ui.button("String").clicked() { self.data = Data::String("".into()); }
            }).response;
        res |= self.name.show_editor(&mut (), ui);
        res | match &mut self.data {
            Data::UInt(n) => ui.add(DragValue::new(n).hexadecimal(1, false, false)),
            Data::String(s) => s.show_editor(&mut (), ui),
            Data::Float(n) => ui.add(DragValue::new(n)),
            Data::Bool(b) => ui.checkbox(b, ""),
            Data::Int(n) => ui.add(DragValue::new(n)),
        }
    }
}

impl Editor for CPF {
    type EditorState = ();

    fn new_editor(&self, _context: &Context) -> Self::EditorState {}

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let ires = ui.horizontal_wrapped(|ui| {
            let mut res = ComboBox::new("cpf_version", "Version ")
                .selected_text(if matches!(self.version, CPFVersion::XML(_, _)) {
                    "XML"
                } else {
                    "CPF"
                })
                .show_ui(ui, |ui| {
                    if ui.button("XML").clicked() {
                        self.version = CPFVersion::XML(DataType::String, None);
                    }
                    if ui.button("CPF").clicked() {
                        self.version = CPFVersion::CPF(2);
                    }
                }).response;
            match &mut self.version {
                CPFVersion::CPF(version) => {
                    res |= ui.add(DragValue::new(version))
                }
                CPFVersion::XML(data_type, version) => {
                    ComboBox::new("data_type", "XML data type")
                        .selected_text(format!("{data_type:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(data_type, DataType::String, "String");
                            ui.selectable_value(data_type, DataType::UInt, "UInt");
                        });
                    let mut has_version = version.is_some();
                    ui.checkbox(&mut has_version, "Has version attribute");
                    match (version, has_version) {
                        (Some(v), true) => {
                            ui.add(DragValue::new(v));
                        }
                        (v @ _, true) => {
                            *v = Some(7);
                        }
                        (v @ Some(_), false) => {
                            *v = None;
                        }
                        _ => {}
                    }
                }
            }
            res
        });
        let mut res = ires.response | ires.inner;

        ui.separator();

        res |= self.entries.show_editor(&mut VecEditorState {
            columns: 3,
            elem_states: vec![(); self.entries.len()],
        }, ui);

        res
    }
}
