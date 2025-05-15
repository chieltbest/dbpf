use eframe::egui::{ComboBox, Context, DragValue, Grid, Response, Ui};
use dbpf::internal_file::behaviour::behaviour_function::{BehaviourFunction, Goto, Instruction, Signature};
use crate::editor::{Editor, VecEditorState, VecEditorStateStorage};
use crate::editor::r#enum::{EnumEditor, EnumEditorState};


impl EnumEditor for Goto {
    type KnownEnum = Goto;

    fn from_known(known_enum: &Self::KnownEnum) -> Self {
        *known_enum
    }

    fn from_int_string(string: &String) -> Option<Self>
    where
        Self: Sized,
    {
        u16::from_str_radix(string.trim_start_matches("0x"), 16)
            .map(Self::Instr)
            .ok()
    }

    fn known_name(known_enum: &Self::KnownEnum) -> String {
        format!("{known_enum:?}")
    }

    fn full_name(&self) -> String {
        match self {
            Goto::Error |
            Goto::True |
            Goto::False => format!("{self:?}"),
            Goto::Instr(u) => format!("{u:x?}"),
        }
    }

    fn known_hover_string(known_enum: &Self::KnownEnum) -> String {
        match known_enum {
            Goto::Error => "0xFD, 0xFFFC".to_string(),
            Goto::True => "0xFE, 0xFFFD".to_string(),
            Goto::False => "0xFF, 0xFFFE".to_string(),
            Goto::Instr(n) => format!("{n:X?}"),
        }
    }

    fn hover_string(&self) -> Option<String> {
        None
    }

    fn search_strings(known_enum: &Self::KnownEnum) -> Vec<String> {
        vec![format!("{known_enum:?}")]
    }

    fn all_known() -> impl Iterator<Item=Self::KnownEnum> {
        vec![Self::Error, Self::False, Self::True].into_iter()
    }
}

impl Editor for Goto {
    type EditorState = EnumEditorState;

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        self.show_enum_editor(state, ui)
    }
}

impl Editor for Instruction {
    type EditorState = <Goto as Editor>::EditorState;

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let mut res = ui.add(DragValue::new(&mut self.opcode).hexadecimal(1, false, false));
        res |= self.true_target.show_editor(state, ui);
        res |= self.false_target.show_editor(state, ui);
        res |= ui.add(DragValue::new(&mut self.node_version).hexadecimal(1, false, false));

        res | ui.horizontal(|ui| {
            ui.label("(");
            ui.style_mut().spacing.interact_size.x = 25.0;
            let res = self.operands.iter_mut().map(|u| {
                ui.add(DragValue::new(u).hexadecimal(2, false, false))
            }).reduce(|r1, r2| r1 | r2).unwrap();
            ui.label(")");
            res
        }).inner
    }
}

impl Editor for BehaviourFunction {
    type EditorState = VecEditorState<Instruction>;

    fn new_editor(&self, _context: &Context) -> Self::EditorState {
        VecEditorState {
            columns: 5,
            storage: VecEditorStateStorage::Shared(Default::default()),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = Grid::new("bhav edit grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("filename");
                let mut res = self.name.name.show_editor(&mut (), ui);
                ui.end_row();

                ui.label("signature");
                res |= ComboBox::from_id_salt("signature")
                    .selected_text(format!("{:?}", self.signature))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.signature, Signature::V0, "V0");
                        ui.selectable_value(&mut self.signature, Signature::V1, "V1");
                        ui.selectable_value(&mut self.signature, Signature::V2, "V2");
                        ui.selectable_value(&mut self.signature, Signature::V3, "V3");
                        ui.selectable_value(&mut self.signature, Signature::V4, "V4");
                        ui.selectable_value(&mut self.signature, Signature::V5, "V5");
                        ui.selectable_value(&mut self.signature, Signature::V6, "V6");
                        ui.selectable_value(&mut self.signature, Signature::V7, "V7");
                        ui.selectable_value(&mut self.signature, Signature::V8, "V8");
                        ui.selectable_value(&mut self.signature, Signature::V9, "V9");
                    }).response;
                ui.end_row();

                macro_rules! drag {
                    ($key:ident) => {
                        ui.label(stringify!($key));
                        res |= ui.add(DragValue::new(&mut self.$key).hexadecimal(1, false, false));
                        ui.end_row();
                    };
                }

                drag!(tree_type);
                drag!(num_parameters);
                drag!(num_locals);
                drag!(header_flags);

                drag!(tree_version);

                drag!(cache_flags);

                res
            }).inner;

        res | self.instructions.show_editor(state, ui)
    }
}
