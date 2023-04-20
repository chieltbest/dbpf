use std::cell::RefCell;
use std::rc::Rc;
use eframe::egui::{DragValue, Ui};
use dbpf::IndexMinorVersion;
use dbpf::internal_file::sim_outfits::{Entry, SimOutfits};
use crate::editor::{Editor, VecEditorState};
use crate::editor::file_type::DBPFFileTypeEditorState;

impl Editor for Entry {
    type EditorState = Rc<RefCell<DBPFFileTypeEditorState>>;

    fn new_editor(&self) -> Self::EditorState {
        Rc::new(RefCell::new(self.type_id.new_editor()))
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        self.type_id.show_editor(&mut state.borrow_mut(), ui);
        ui.add(DragValue::new(&mut self.group_id)
            .hexadecimal(8, false, true));
        ui.add(DragValue::new(&mut self.instance_id.id)
            .hexadecimal(8, false, true));
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SimOutfitsEditorState {
    file_type_chooser_state: Rc<RefCell<DBPFFileTypeEditorState>>,
}

impl Editor for SimOutfits {
    type EditorState = SimOutfitsEditorState;

    fn new_editor(&self) -> Self::EditorState {
        Default::default()
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Index version");
            ui.selectable_value(&mut self.version, IndexMinorVersion::V1, "V1");
            ui.selectable_value(&mut self.version, IndexMinorVersion::V2, "V2");
        });

        ui.separator();

        let states_vec = vec![state.file_type_chooser_state.clone(); self.entries.len()];
        let mut vec_state = VecEditorState {
            columns: 3,
            elem_states: states_vec,
        };

        self.entries.show_editor(
            &mut vec_state,
            ui);
    }
}
