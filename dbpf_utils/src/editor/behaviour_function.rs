use std::cmp::max;
use std::collections::{BTreeSet, VecDeque};
use eframe::egui::{Color32, ComboBox, Context, DragValue, Grid, Pos2, Response, Ui};
use egui_snarl::{InPin, InPinId, NodeId, OutPin, OutPinId, Snarl};
use egui_snarl::ui::{PinInfo, PinPlacement, SnarlPin, SnarlStyle, SnarlViewer, WireLayer, WireStyle};
use dbpf::internal_file::behaviour::behaviour_function::{BehaviourFunction, Goto, Instruction, Signature};
use crate::editor::Editor;
use crate::editor::r#enum::{EnumEditor, EnumEditorState};

impl EnumEditor for Goto {
    type KnownEnum = Goto;

    fn from_known(known_enum: &Self::KnownEnum) -> Self {
        *known_enum
    }

    fn from_string(string: &String) -> Option<Self>
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

#[derive(Debug, Default)]
pub struct BhavEditorState {
    snarl: Snarl<Instruction>,
    enum_editor_state: EnumEditorState,
}

#[derive(Debug)]
struct BhavViewer<'a, 'b> {
    _bhav: &'a mut BehaviourFunction,
    enum_editor_state: &'b mut EnumEditorState,
}

impl<'a, 'b> SnarlViewer<Instruction> for BhavViewer<'a, 'b> {
    fn title(&mut self, _node: &Instruction) -> String {
        "Instruction".to_string()
    }

    fn inputs(&mut self, _node: &Instruction) -> usize {
        1
    }

    fn show_input(&mut self, _pin: &InPin, _ui: &mut Ui, _scale: f32, _snarl: &mut Snarl<Instruction>) -> impl SnarlPin + 'static {
        PinInfo::circle()
    }

    fn outputs(&mut self, _node: &Instruction) -> usize {
        2
    }

    fn show_output(&mut self, pin: &OutPin, ui: &mut Ui, _scale: f32, snarl: &mut Snarl<Instruction>) -> impl SnarlPin + 'static {
        let (target, color) = if pin.id.output == 0 {
            (&mut snarl[pin.id.node].true_target, Color32::GREEN)
        } else {
            (&mut snarl[pin.id.node].false_target, Color32::RED)
        };
        target.show_editor(self.enum_editor_state, ui);
        PinInfo::circle().with_fill(color).with_wire_color(color)
    }

    fn has_body(&mut self, _node: &Instruction) -> bool {
        true
    }

    fn show_body(&mut self, node: NodeId, _inputs: &[InPin], _outputs: &[OutPin], ui: &mut Ui, scale: f32, snarl: &mut Snarl<Instruction>) {
        let instr = &mut snarl[node];
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.label("opcode");
                ui.add(DragValue::new(&mut instr.opcode).hexadecimal(1, false, false));
                ui.label("version");
                ui.add(DragValue::new(&mut instr.node_version).hexadecimal(1, false, false));
            });

            ui.style_mut().spacing.interact_size.x = 25.0 * scale;
            ui.horizontal(|ui| {
                for u in &mut instr.operands[..8] {
                    ui.add(DragValue::new(u).hexadecimal(2, false, false));
                }
            });
            ui.horizontal(|ui| {
                for u in &mut instr.operands[8..] {
                    ui.add(DragValue::new(u).hexadecimal(2, false, false));
                }
            });
        });
    }
}

impl Editor for BehaviourFunction {
    type EditorState = BhavEditorState;

    fn new_editor(&self, _context: &Context) -> Self::EditorState {
        let mut snarl = Snarl::new();

        let mut unopened: BTreeSet<usize> = (0..self.instructions.len()).collect();

        let mut sorted_instr_groups = vec![];

        while let Some(first) = unopened.pop_first() {
            let mut open_list = VecDeque::from([(first, 0, 0)]);
            let mut sorted_instr_ids = vec![];

            while let Some((id, x, y)) = open_list.pop_back() {
                sorted_instr_ids.push((id, x, y));

                let instruction = &self.instructions[id];

                let targets = [instruction.true_target, instruction.false_target]
                    .map(|t| match t {
                        Goto::Instr(i) => unopened
                            .remove(&(i as usize))
                            .then_some(i as usize),
                        _ => None,
                    });
                match targets {
                    [Some(t), Some(f)] => {
                        open_list.push_back((t, x + 1, y + 1));
                        open_list.push_back((f, x - 1, y + 1));
                    }
                    [Some(i), _] | [_, Some(i)] => {
                        open_list.push_back((i, x, y + 1));
                    }
                    _ => {}
                }
            }

            sorted_instr_groups.push(sorted_instr_ids);
        }

        let mut occupied_cells = BTreeSet::new();
        let mut node_positons = self.instructions.iter()
            .map(|_i| Pos2::new(0.0, 0.0))
            .collect::<Vec<_>>();
        let mut max_position = 0;

        for group in sorted_instr_groups.into_iter() {
            let group_min = group.iter()
                .map(|(_, x, _)| x)
                .min()
                .unwrap();

            let group_x_offset = max_position - group_min;

            for (id, x, mut y) in group {
                let x = x + group_x_offset;
                while occupied_cells.contains(&(x, y)) {
                    y += 1;
                }

                occupied_cells.insert((x - 1, y));
                occupied_cells.insert((x, y));
                occupied_cells.insert((x + 1, y));
                node_positons[id] = Pos2::new(200.0 * x as f32, 150.0 * y as f32);

                max_position = max(max_position, x + 2)
            }
        }

        for (instr, pos) in self.instructions.iter()
            .zip(node_positons) {
            snarl.insert_node(pos, instr.clone());
        }

        for (i, instr) in self.instructions.iter().enumerate() {
            if let Goto::Instr(target) = instr.true_target {
                snarl.connect(OutPinId {
                    node: NodeId(i),
                    output: 0,
                }, InPinId {
                    node: NodeId(target as usize),
                    input: 0,
                });
            }
            if let Goto::Instr(target) = instr.false_target {
                snarl.connect(OutPinId {
                    node: NodeId(i),
                    output: 1,
                }, InPinId {
                    node: NodeId(target as usize),
                    input: 0,
                });
            }
        }

        BhavEditorState {
            snarl,
            enum_editor_state: EnumEditorState::default(),
        }
    }

    fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
        let res = Grid::new("bhav edit grid")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("filename");
                let mut res = self.name.name.show_editor(&mut 500.0, ui);
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

        let mut snarl_style = SnarlStyle::new();
        snarl_style.wire_style = Some(WireStyle::Bezier5 {});
        snarl_style.wire_layer = Some(WireLayer::BehindNodes);
        snarl_style.wire_width = Some(5.0);
        snarl_style.wire_frame_size = Some(50.0);
        snarl_style.upscale_wire_frame = Some(true);
        snarl_style.downscale_wire_frame = Some(true);
        snarl_style.pin_placement = Some(PinPlacement::Edge {});

        state.snarl.show(&mut BhavViewer {
            _bhav: self,
            enum_editor_state: &mut state.enum_editor_state,
        }, &snarl_style, "bhav snarl", ui);

        res
    }
}
