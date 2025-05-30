use crate::editor::{Editor, VecEditorState, VecEditorStateStorage};
use dbpf::internal_file::cpf::{CPFVersion, Data, Item, Reference, XMLDataType, CPF};
use eframe::egui::{ComboBox, DragValue, Response, Ui};

mod property_set;
mod binary_index;

impl Editor for Item {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = ComboBox::from_id_salt("data_type")
            .width(60.0)
            .selected_text(format!("{:?}", self.data.get_type()))
            .show_ui(ui, |ui| {
                if ui.button("UInt").clicked() { self.data = Data::UInt(0); }
                if ui.button("Int").clicked() { self.data = Data::Int(0); }
                if ui.button("Float").clicked() { self.data = Data::Float(0.0); }
                if ui.button("Bool").clicked() { self.data = Data::Bool(false); }
                if ui.button("String").clicked() { self.data = Data::String("".into()); }
            }).response;
        res |= self.name.show_editor(&mut 300.0, ui);
        res | match &mut self.data {
            Data::UInt(n) => ui.add(DragValue::new(n).hexadecimal(1, false, false)),
            Data::String(s) => s.show_editor(&mut 300.0, ui),
            Data::Float(n) => ui.add(DragValue::new(n)),
            Data::Bool(b) => ui.checkbox(b, ""),
            Data::Int(n) => ui.add(DragValue::new(n)),
        }
    }
}

impl Editor for CPF {
    type EditorState = ();

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
                        self.version = CPFVersion::XML(XMLDataType::String, None);
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
                            ui.selectable_value(data_type, XMLDataType::String, "String");
                            ui.selectable_value(data_type, XMLDataType::UInt, "UInt");
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
            storage: VecEditorStateStorage::Shared(()),
        }, ui);

        res
    }
}

fn reference_edit_fn(name: &str, value: &mut Reference, ui: &mut Ui) -> Response {
    if !name.is_empty() {
        ui.label(name);
    }

    let res = ui.horizontal(|ui| {
        let mut cur_type = match value {
            Reference::Idx(_) => 0,
            Reference::TGI(_, _, _) => 1,
        };
        let mut res = ComboBox::from_id_salt(name)
            .width(50.0)
            .selected_text(match value {
                Reference::Idx(_) => "idx",
                Reference::TGI(_, _, _) => "TGI",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cur_type, 0, "idx");
                ui.selectable_value(&mut cur_type, 1, "TGI");
            }).response;

        match (value.clone(), cur_type) {
            (Reference::Idx(_), 1) => {
                *value = Reference::TGI(0, 0, 0);
            }
            (Reference::TGI(_, _, _), 0) => {
                *value = Reference::Idx(0);
            }
            _ => {}
        }

        match value {
            Reference::Idx(idx) => {
                res |= ui.add(DragValue::new(idx).hexadecimal(1, false, false));
            }
            Reference::TGI(t, g, i) => {
                res |= ui.add(DragValue::new(t).hexadecimal(1, false, false));
                res |= ui.add(DragValue::new(g).hexadecimal(1, false, false));
                res |= ui.add(DragValue::new(i).hexadecimal(1, false, false));
            }
        }

        res
    });

    ui.end_row();

    res.inner
}
