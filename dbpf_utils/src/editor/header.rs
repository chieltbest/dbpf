use eframe::egui::{ComboBox, DragValue, Response, Ui};
use dbpf::{IndexVersion, Timestamp, UserVersion, V1Minor, V2Minor, V3Minor, Version};
use crate::editor::Editor;

impl Editor for Version {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let mut cur_selected = match self {
                Version::V1(_) => 0,
                Version::V2(_) => 1,
                Version::V3(_) => 2,
            };

            let mut res = ComboBox::from_id_salt("version")
                .width(50.0)
                .selected_text(match self {
                    Version::V1(_) => "V1",
                    Version::V2(_) => "V2",
                    Version::V3(_) => "V3",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cur_selected, 0, "V1");
                    ui.selectable_value(&mut cur_selected, 1, "V2");
                    ui.selectable_value(&mut cur_selected, 2, "V3");
                }).response;
            if res.changed() {
                *self = match cur_selected {
                    0 => Version::V1(V1Minor::M2),
                    1 => Version::V2(V2Minor::M1),
                    _ => Version::V3(V3Minor::M0),
                };
            }
            match self {
                Version::V1(m) => {
                    res |= ComboBox::from_id_salt("version minor")
                        .width(50.0)
                        .selected_text(format!("{m:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(m, V1Minor::M0, "M0");
                            ui.selectable_value(m, V1Minor::M1, "M1");
                            ui.selectable_value(m, V1Minor::M2, "M2");
                        }).response;
                }
                Version::V2(m) => {
                    res |= ComboBox::from_id_salt("version minor")
                        .width(50.0)
                        .selected_text(format!("{m:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(m, V2Minor::M0, "M0");
                            ui.selectable_value(m, V2Minor::M1, "M1");
                        }).response;
                }
                Version::V3(m) => {
                    res |= ComboBox::from_id_salt("version minor")
                        .width(50.0)
                        .selected_text(format!("{m:?}"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(m, V3Minor::M0, "M0");
                        }).response;
                }
            }
            res
        }).inner
    }
}

impl Editor for IndexVersion {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = ComboBox::from_id_salt("index version")
            .selected_text(format!("{self:?}"))
            .show_ui(ui, |ui| {
                let res = ui.selectable_value(self, IndexVersion::Default, "Default");
                res | ui.selectable_value(self, IndexVersion::Spore, "Spore")
            });
        if let Some(inner) = res.inner {
            res.response | inner
        } else {
            res.response
        }
    }
}

impl Editor for UserVersion {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let res = ui.add(DragValue::new(&mut self.major).prefix("v"));
            res | ui.add(DragValue::new(&mut self.minor).prefix("."))
        }).inner
    }
}

impl Editor for Timestamp {
    type EditorState = ();

    fn show_editor(&mut self, _state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            let dres = ui.add(DragValue::new(&mut self.0));

            let datetime = chrono::DateTime::from_timestamp(self.0 as i64, 0).unwrap();
            let mut naive = datetime.date_naive();
            let res = ui.add(egui_extras::DatePickerButton::new(&mut naive));

            if res.changed() {
                let new_datetime = naive.and_hms_opt(0, 0, 0).unwrap().and_utc();
                if let Ok(new_timestamp) = u32::try_from(new_datetime.timestamp()) {
                    self.0 = new_timestamp;
                }
            }

            dres | res
        }).inner
    }
}
