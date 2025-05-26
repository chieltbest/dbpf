#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// TODO add open with resource tgi arguments
// TODO add resource

use binrw::{BinRead, BinResult};
use clap::Parser;
use dbpf::filetypes::{DBPFFileType, KnownDBPFFileType};
use dbpf::internal_file::CompressionError;
use dbpf::{CompressionType, DBPFFile, IndexEntry};
use eframe::egui::{Button, Color32, Context, DragValue, Id, Label, Rect, Response, ScrollArea, Sense, Stroke, Style, Ui, Visuals, WidgetText};
use eframe::{egui, App, Frame, Storage};
use egui_dock::{DockState, Node, NodeIndex, Split, TabIndex, TabViewer};
use egui_extras::Column;
use egui_memory_editor::option_data::MemoryEditorOptions;
use egui_memory_editor::MemoryEditor;
use futures::channel::oneshot;
use rfd::FileHandle;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::io::{Cursor, Read, Seek, Write};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use tracing::error;

use dbpf_utils::editor::{editor_supported, DecodedFileEditorState, Editor};
use dbpf_utils::{async_execute, graphical_application_main};

enum EditorType {
    HexEditor(MemoryEditor),
    DecodedEditor(DecodedFileEditorState),
    Error(CompressionError),
}

impl Debug for EditorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(match self {
            EditorType::HexEditor(_) => "HexEditor",
            EditorType::DecodedEditor(_) => "DecodedEditor",
            EditorType::Error(_) => "Error",
        }).field(match self {
            EditorType::HexEditor(hex) => &hex.options,
            EditorType::DecodedEditor(decoded) => decoded,
            EditorType::Error(err) => err,
        }).finish()
    }
}

impl Default for EditorType {
    fn default() -> Self {
        Self::HexEditor(MemoryEditor::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EntryEditorTab {
    #[serde(skip)]
    state: EditorType,
    #[serde(skip)]
    data: Weak<RefCell<IndexEntry>>,

    #[serde(skip)]
    id: usize,

    // used for (de)serialising
    #[serde(default)]
    is_hex_editor: bool,
    #[serde(default)]
    index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
enum YaPeTab {
    File,
    Entry(EntryEditorTab),
}

#[derive(Debug, Serialize, Deserialize)]
enum SplitDirection {
    Horizontal,
    Vertical,
    Tabs,
}

#[derive(Debug, Serialize, Deserialize)]
struct RootNodeState {
    fraction: f32,
    split: SplitDirection,
}

impl Default for RootNodeState {
    fn default() -> Self {
        Self {
            fraction: 0.5,
            split: SplitDirection::Vertical,
        }
    }
}

fn file_type_ser<S>((ft, e): &(DBPFFileType, bool), ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    (ft.code(), e).serialize(ser)
}
fn file_type_deser<'de, D>(deser: D) -> Result<(DBPFFileType, bool), D::Error>
where
    D: Deserializer<'de>,
{
    <(u32, bool)>::deserialize(deser).map(|(ft, e)| (ft.into(), e))
}
fn file_type_default() -> (DBPFFileType, bool) {
    (DBPFFileType::Known(KnownDBPFFileType::TextureResource), false)
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct YaPeAppData {
    #[serde(default)]
    memory_editor_options: MemoryEditorOptions,

    #[serde(default)]
    open_file_path: Option<PathBuf>,

    #[serde(default)]
    highlight_index: Option<usize>,

    #[serde(default = "file_type_default", serialize_with = "file_type_ser", deserialize_with = "file_type_deser")]
    type_filter: (DBPFFileType, bool),
    #[serde(skip)]
    type_filter_state: <DBPFFileType as Editor>::EditorState,

    #[serde(skip)]
    open_file: Option<(Cursor<Vec<u8>>, BinResult<DBPFFile>, Vec<Rc<RefCell<IndexEntry>>>)>,

    #[serde(skip)]
    open_new_tab_index: Option<usize>,
    #[serde(skip)]
    open_new_hex_tab_index: Option<usize>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct YaPeApp {
    ui_scale: f32,
    #[serde(default)]
    dark_mode_preference: Option<bool>,

    dock_state: DockState<YaPeTab>,
    #[serde(default)]
    root_node_state: RootNodeState,

    #[serde(default)]
    data: YaPeAppData,

    #[serde(skip)]
    next_tab_id: usize,

    /// Rusty file dialog async read file picker
    #[serde(skip)]
    file_picker: Option<oneshot::Receiver<Option<(Vec<u8>, PathBuf)>>>,
    /// Rusty file dialog async save as file picker
    #[serde(skip)]
    save_file_picker: Option<oneshot::Receiver<Option<FileHandle>>>,
}

impl Default for YaPeApp {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            dark_mode_preference: None,

            dock_state: DockState::new(vec![YaPeTab::File]),
            root_node_state: Default::default(),

            data: Default::default(),

            next_tab_id: 0,

            file_picker: None,
            save_file_picker: None,
        }
    }
}

impl EntryEditorTab {
    fn show<R: Read + Seek>(&mut self, ui: &mut Ui, reader: &mut R) {
        if let Some(data) = self.data.upgrade() {
            match &mut self.state {
                EditorType::Error(err) => {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.label(format!("{err:?}"));
                    });
                }
                EditorType::HexEditor(editor) => {
                    let mut data_ref = data.borrow_mut();
                    let data = data_ref.data(reader).unwrap().decompressed().unwrap();
                    if let Ok(mut str) = String::from_utf8(data.data.clone()) {
                        if !self.is_hex_editor {
                            ui.centered_and_justified(|ui|
                                ScrollArea::vertical().show(ui, |ui| {
                                    if ui.code_editor(&mut str).changed() {
                                        data.data = str.into_bytes();
                                    }
                                })
                            );
                            return;
                        }
                    }
                    editor.draw_editor_contents(
                        ui,
                        data,
                        |mem, addr| Some(mem.data[addr]),
                        |mem, addr, byte| mem.data[addr] = byte,
                    );
                }
                EditorType::DecodedEditor(state) => {
                    let mut data_ref = data.borrow_mut();
                    let decoded = data_ref.data(reader).unwrap().decoded().unwrap().unwrap();
                    decoded.show_editor(state, ui);
                }
            }
        }
    }
}

impl YaPeAppData {
    fn show_index(&mut self, ui: &mut Ui) {
        let mut open_index = None;
        let mut open_hex_index = None;

        match &mut self.open_file {
            None => {}
            Some((_, Err(err), _entries)) => {
                ui.colored_label(Color32::RED, err.to_string());
            }
            Some((_, Ok(file), entries)) => {
                ui.collapsing("header", |ui| {
                    let header = &mut file.header;
                    egui::Grid::new("header grid")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("version");
                            header.version.show_editor(&mut (), ui);
                            ui.end_row();

                            ui.label("index version");
                            header.index_version.show_editor(&mut (), ui);
                            ui.end_row();

                            ui.label("user version");
                            header.user_version.show_editor(&mut (), ui);
                            ui.end_row();

                            ui.label("flags");
                            ui.add(egui::DragValue::new(&mut header.flags).hexadecimal(1, false, false));
                            ui.end_row();

                            ui.label("created");
                            ui.push_id("created", |ui| {
                                header.created.show_editor(&mut (), ui);
                            });
                            ui.end_row();

                            ui.label("modified");
                            ui.push_id("modified", |ui| {
                                header.modified.show_editor(&mut (), ui);
                            });
                            ui.end_row();
                        });
                });

                let mut delete_index = None;

                let style_mut = ui.style_mut();
                let button_height = style_mut.spacing.interact_size.y;
                style_mut.visuals.selection.bg_fill = if style_mut.visuals.dark_mode {
                    Color32::from_gray(16)
                } else {
                    Color32::LIGHT_GRAY
                };
                style_mut.visuals.selection.stroke = if style_mut.visuals.dark_mode {
                    Stroke {
                        color: Color32::WHITE,
                        ..Default::default()
                    }
                } else {
                    Stroke {
                        color: Color32::BLACK,
                        ..Default::default()
                    }
                };

                let filtered_entries = entries.iter()
                    .enumerate()
                    .filter(|(_i, e)| {
                        !self.type_filter.1 || (*e).borrow().type_id == self.type_filter.0
                    }).collect::<Vec<_>>();
                let filtered_count = filtered_entries.len();

                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::exact(20.0))
                    .column(Column::auto()
                        .at_least(100.0)
                        .clip(true))
                    .column(Column::auto()
                        .at_least(80.0)
                        .clip(true))
                    .column(Column::auto()
                        .at_least(150.0)
                        .clip(true))
                    .column(Column::remainder()
                        .at_least(100.0)
                        .clip(true))
                    .min_scrolled_height(100.0)
                    .max_scroll_height(f32::MAX)
                    .header(40.0, |mut row| {
                        row.col(|_ui| {});
                        row.col(|ui| {
                            let (t, e) = &mut self.type_filter;
                            ui.horizontal(|ui| {
                                ui.label("Type");
                                ui.checkbox(e, "filter")
                                    .on_hover_text("type filter enabled");
                            });
                            if t.show_editor(&mut self.type_filter_state, ui).changed() {
                                *e = true;
                            }
                        });
                        row.col(|ui| { ui.label("Group"); });
                        row.col(|ui| { ui.label("Instance"); });
                        row.col(|ui| { ui.label("Compression"); });
                    })
                    .body(|body| {
                        body.rows(button_height, filtered_count,
                                  |mut row| {
                                      let (i, entry_rc) = filtered_entries[row.index()];
                                      let mut entry = entry_rc.borrow_mut();

                                      let mut sense_fun = |ui: &mut Ui, res: Response, clicked: bool, entry: &IndexEntry| {
                                          let interact_res = ui.interact(
                                              Rect::everything_right_of(res.rect.right()),
                                              Id::from(format!("row_interact_{i}")),
                                              Sense::click(),
                                          );
                                          if interact_res.clicked() || (clicked && res.clicked()) {
                                              open_index = Some(i);
                                          }
                                          (res | interact_res).context_menu(|ui| {
                                              if ui.button("Filter on type").clicked() {
                                                  self.type_filter = (entry.type_id, true);
                                                  ui.close_menu();
                                              }
                                              if ui.button("Open hex editor").clicked() {
                                                  open_hex_index = Some(i);
                                                  ui.close_menu();
                                              }
                                          });
                                      };

                                      let selected = self.highlight_index.is_some_and(|hi| hi == i);
                                      row.set_selected(selected);

                                      row.col(|ui| {
                                          ui.button("🗑").clicked().then(|| {
                                              delete_index = Some(i);
                                          });
                                      });
                                      row.col(|ui| {
                                          let t = entry.type_id;
                                          let res = ui.horizontal_centered(|ui| {
                                              ui.add(Label::new(t.properties().map_or_else(
                                                  || format!("{:08X}", t.code()),
                                                  |prop| prop.abbreviation.to_string()))
                                                  .sense(Sense::click())
                                                  .selectable(false))
                                                  .on_hover_text(format!(
                                                      "{}\n{:08X}",
                                                      t.full_name(),
                                                      t.code()))
                                          });
                                          sense_fun(ui, res.inner, true, &entry);
                                      });
                                      row.col(|ui| {
                                          let res = ui.add(DragValue::new(&mut entry.group_id)
                                              .hexadecimal(8, false, true));
                                          sense_fun(ui, res, false, &entry);
                                      });
                                      row.col(|ui| {
                                          let res = ui.add(DragValue::new(&mut entry.instance_id)
                                              .hexadecimal(8, false, true));
                                          sense_fun(ui, res, false, &entry);
                                      });
                                      row.col(|ui| {
                                          let res = egui::ComboBox::from_id_salt(
                                              format!("{:?}{}{}", entry.type_id, entry.group_id, entry.instance_id))
                                              .selected_text(format!("{:?}", entry.compression))
                                              .width(110.0)
                                              .show_ui(ui, |ui| {
                                                  ui.selectable_value(&mut entry.compression, CompressionType::Uncompressed, "Uncompressed");
                                                  ui.selectable_value(&mut entry.compression, CompressionType::RefPack, "RefPack");
                                                  ui.selectable_value(&mut entry.compression, CompressionType::ZLib, "ZLib");
                                                  ui.selectable_value(&mut entry.compression, CompressionType::Streamable, "Streamable");
                                                  ui.selectable_value(&mut entry.compression, CompressionType::Deleted, "Deleted");
                                              });
                                          sense_fun(ui, res.response, false, &entry);
                                      });
                                  });
                    });

                self.open_new_tab_index = open_index;
                if open_index.is_some() {
                    self.highlight_index = open_index;
                }
                self.open_new_hex_tab_index = open_hex_index;
                if open_hex_index.is_some() {
                    self.highlight_index = open_hex_index;
                }

                if let Some(delete_i) = delete_index {
                    entries.remove(delete_i);
                    self.highlight_index = None;
                }
            }
        }
    }
}

impl TabViewer for YaPeAppData {
    type Tab = YaPeTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            YaPeTab::File => {
                self.open_file_path
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|p| p.to_string_lossy().into())
                    .unwrap_or("Index".to_string()).into()
            },
            YaPeTab::Entry(entry) => {
                if let Some(index) = entry.data.upgrade() {
                    index.borrow().type_id.full_name().into()
                } else {
                    "Unknown".into()
                }
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            YaPeTab::File => self.show_index(ui),
            YaPeTab::Entry(entry) => {
                if let Some((cur, _file, _entries)) = &mut self.open_file {
                    entry.show(ui, cur);
                }
            }
        }
    }

    fn id(&mut self, tab: &mut Self::Tab) -> Id {
        let id = Id::new(self.title(tab).text());
        match tab {
            YaPeTab::File => id,
            YaPeTab::Entry(e) => {
                id.with(e.id)
            }
        }
    }

    fn on_tab_button(&mut self, _tab: &mut Self::Tab, response: &Response) {
        if matches!(_tab, Self::Tab::File) {
            if let Some(p) = &self.open_file_path {
                response.clone().on_hover_text(p.to_string_lossy());
            }
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            YaPeTab::File => false,
            YaPeTab::Entry(_) => true,
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            YaPeTab::File => false,
            YaPeTab::Entry(entry) => entry.data.strong_count() == 0,
        }
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        match _tab {
            YaPeTab::File => [true, false],
            YaPeTab::Entry(_) => [true, true]
        }
    }
}

impl YaPeApp {
    fn new(cc: &eframe::CreationContext<'_>, args: Args) -> Self {
        if let Some(storage) = cc.storage {
            let mut new: YaPeApp = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            cc.egui_ctx.set_pixels_per_point(new.ui_scale);
            if let Some(dark) = new.dark_mode_preference {
                new.set_dark_mode(dark, &cc.egui_ctx);
            }

            if let Some(path) = args.files.first() {
                new.open_file(path.clone());
            } else if let Some(path) = new.data.open_file_path.clone() {
                new.open_file(path);

                if let Some((data, _file, rc_entries)) = &mut new.data.open_file {
                    // TODO clean up this mess
                    let id_rc = Rc::new(RefCell::from(&mut new.next_tab_id));
                    let data_rc = Rc::new(RefCell::from(data));

                    new.dock_state = new.dock_state.filter_map_tabs(|tab| {
                        match tab {
                            YaPeTab::File => Some(YaPeTab::File),
                            YaPeTab::Entry(entry) => {
                                let hex_editor = entry.is_hex_editor;
                                entry.index.and_then(|i|
                                    Self::open_index(*id_rc.borrow_mut(), *data_rc.borrow_mut(), rc_entries, i, hex_editor, &cc.egui_ctx)
                                        .map(YaPeTab::Entry))
                            }
                        }
                    });
                }
            }

            return new;
        }

        Self::default()
    }

    fn set_dark_mode(&mut self, dark: bool, ctx: &Context) {
        self.dark_mode_preference = Some(dark);
        ctx.set_style(Style {
            visuals: if dark {
                Visuals::dark()
            } else {
                Visuals::light()
            },
            ..Default::default()
        })
    }

    fn open_bytes(&mut self, bytes: Vec<u8>) {
        let mut cursor = Cursor::new(bytes);
        let mut parsed = DBPFFile::read(&mut cursor);
        let rc_index = match &mut parsed {
            Ok(file) => {
                std::mem::take(&mut file.index)
                    .into_iter()
                    .map(|entry| {
                        Rc::new(RefCell::new(entry))
                    })
                    .collect()
            }
            Err(e) => {
                error!(?e);
                vec![]
            }
        };

        self.data.open_file = Some((cursor, parsed, rc_index));

        // self.dock_state = DockState::new(vec![YaPeTab::File]);
    }

    fn open_file(&mut self, path: PathBuf) {
        let Ok(bytes) = fs::read(path.clone())
            .inspect_err(|e| error!(?e)) else {
            return;
        };
        self.open_bytes(bytes);
        self.data.open_file_path = Some(path);
    }

    #[must_use]
    fn open_index<R: Read + Seek>(next_tab_id: &mut usize, reader: &mut R, rc_entries: &Vec<Rc<RefCell<IndexEntry>>>, index: usize, hex_editor: bool, ui_ctx: &Context) -> Option<EntryEditorTab> {
        let id = *next_tab_id;
        *next_tab_id = next_tab_id.wrapping_add(1);
        let index_entry = &rc_entries.get(index)?;
        let mut entry_ref = index_entry.borrow_mut();
        let file_type = entry_ref.type_id;
        let res = entry_ref.data(reader)
            .map_err(|err| CompressionError::BinResult(err))
            .and_then(|entry| {
                if editor_supported(file_type) && !hex_editor {
                    let decoded = entry.decoded()?.unwrap();
                    Ok(EditorType::DecodedEditor(decoded.new_editor(ui_ctx)))
                } else {
                    let decompressed = entry.decompressed()?;
                    Ok(EditorType::HexEditor(
                        MemoryEditor::new().with_address_range(
                            "", 0..decompressed.data.len())))
                }
            })
            .inspect_err(|err| error!(?err));

        Some(EntryEditorTab {
            state: res.unwrap_or_else(|err| EditorType::Error(err)),
            data: Rc::downgrade(index_entry),
            index: Some(index),
            is_hex_editor: hex_editor,
            id,
        })
    }

    fn open_index_tab(&mut self, index: usize, hex_editor: bool, ui_ctx: &Context) {
        if let Some((cur, _file, rc_entries)) = &mut self.data.open_file {
            let search_rc = &rc_entries[index];
            let open_found = self.dock_state.iter_all_nodes()
                .enumerate()
                .find_map(|(node_i, (surf_i, node))| {
                    node.iter_tabs().enumerate().find_map(|(tab_i, tab)| {
                        match tab {
                            YaPeTab::Entry(t) => {
                                (t.data.ptr_eq(&Rc::downgrade(search_rc)) &&
                                    t.is_hex_editor == hex_editor)
                                    .then_some((surf_i, NodeIndex(node_i), TabIndex(tab_i)))
                            }
                            YaPeTab::File => None,
                        }
                    })
                });
            if let Some((surf_i, node_i, tab_i)) = open_found {
                // tab was already open, just focus it
                self.dock_state.set_active_tab((surf_i, node_i, tab_i));
                // self.dock_state.set_focused_node_and_surface((surf_i, node_i));
                return;
            }

            let Some(tab) = Self::open_index(&mut self.next_tab_id, cur, rc_entries, index, hex_editor, ui_ctx) else { return; };
            let leaf_pos = self.dock_state.iter_all_tabs().skip(1).last().map(|(pos, _tab)| pos);
            if let Some(pos) = leaf_pos {
                if let Some((_i, node)) = self.dock_state.iter_all_nodes_mut().nth(pos.1.0) {
                    node.append_tab(YaPeTab::Entry(tab));
                }
            } else if let Some(focus) = self.dock_state.focused_leaf() {
                match self.root_node_state.split {
                    SplitDirection::Tabs => {
                        if let Some((_, node)) = self.dock_state.iter_all_nodes_mut()
                            .find(|(surface, _)| *surface == focus.0) {
                            node.append_tab(YaPeTab::Entry(tab))
                        }
                    }
                    _ => {
                        self.dock_state.split(
                            focus,
                            match self.root_node_state.split {
                                SplitDirection::Horizontal => Split::Right,
                                _ => Split::Below,
                            },
                            self.root_node_state.fraction,
                            Node::leaf(YaPeTab::Entry(tab)));
                    }
                }
            }
        }
    }

    fn save_bytes<W: Write + Seek>(&mut self, writer: &mut W) -> Result<(), CompressionError> {
        if let Some((cur, file, entries)) = &mut self.data.open_file {
            if let Ok(file) = file {
                file.index = entries.iter().map(|e| e.borrow().clone()).collect();
                file.write(writer, cur)?;
                file.index = vec![];
            }
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn read_file_handle(handle: FileHandle) -> (Vec<u8>, PathBuf) {
    (handle.read().await, handle.path().to_owned())
}

#[cfg(target_arch = "wasm32")]
async fn read_file_handle(handle: FileHandle) -> (Vec<u8>, PathBuf) {
    (handle.read().await, PathBuf::default())
}

impl App for YaPeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if let Some(picker) = &mut self.file_picker {
            if let Ok(Some(file)) = picker.try_recv() {
                self.file_picker = None;
                if let Some((data, path)) = file {
                    self.open_bytes(data);
                    self.data.open_file_path = Some(path);
                }
            }
        }
        if let Some(picker) = &mut self.save_file_picker {
            if let Ok(Some(handle)) = picker.try_recv() {
                self.file_picker = None;
                if let Some(handle) = handle {
                    self.data.open_file_path = Some(handle.path().to_path_buf());
                    let mut buf = Cursor::new(Vec::new());
                    match self.save_bytes(&mut buf) {
                        Err(e) => error!(?e),
                        Ok(_) => {
                            let _ = futures::executor::block_on(handle.write(&buf.into_inner()));
                        }
                    }
                }
            }
        }

        if let Some(root) = self.dock_state.main_surface().root_node() {
            match root {
                Node::Vertical { fraction, .. } => {
                    self.root_node_state.split = SplitDirection::Vertical;
                    self.root_node_state.fraction = *fraction;
                }
                Node::Horizontal { fraction, .. } => {
                    self.root_node_state.split = SplitDirection::Horizontal;
                    self.root_node_state.fraction = *fraction;
                }
                Node::Leaf { tabs, .. } => {
                    if tabs.len() > 1 {
                        self.root_node_state.split = SplitDirection::Tabs;
                    }
                }
                _ => {}
            }
        }

        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let dark_mode = ui.style().visuals.dark_mode;
                    ui.button(if dark_mode { "☀" } else { "🌙" })
                        .on_hover_text(format!("Switch to {} mode", if dark_mode { "light" } else { "dark" }))
                        .clicked().then(|| {
                        self.set_dark_mode(!dark_mode, ctx);
                    });

                    ui.horizontal(|ui| {
                        ui.add(DragValue::new(&mut self.ui_scale)
                            .speed(0.01)
                            .fixed_decimals(1))
                            .on_hover_text("Scale of the interface")
                            .changed().then(|| {
                            ctx.set_pixels_per_point(self.ui_scale);
                        });
                        ui.label("UI Scale");
                    });

                    if ui.button("🗁")
                        .on_hover_text("open file...")
                        .clicked() && self.file_picker.is_none() {
                        let (tx, rx) = oneshot::channel();
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .add_filter("Sims 2/3 package files (.package)", &["package"]);
                        if let Some(path) = self.data.open_file_path.as_ref()
                            .and_then(|p| p.parent()) {
                            dialog = dialog.set_directory(path);
                        }
                        let dialog = dialog.pick_file();
                        async_execute(async move {
                            let file = dialog.await;
                            let _ = if let Some(handle) = file {
                                tx.send(Some(read_file_handle(handle).await))
                            } else {
                                tx.send(None)
                            };
                        });
                        self.file_picker = Some(rx);
                    }

                    if let Some(path) = self.data.open_file_path.clone() {
                        if ui.button("💾")
                            .on_hover_text("save")
                            .clicked() {
                            let mut buf = Cursor::new(Vec::new());
                            match self.save_bytes(&mut buf) {
                                Err(e) => error!(?e),
                                Ok(_) => {
                                    if let Err(e) = fs::write(path, buf.into_inner()) {
                                        error!(?e);
                                    }
                                }
                            }
                        }
                        if ui.button("💾✏")
                            .on_hover_text("save as...")
                            .clicked() && self.save_file_picker.is_none() {
                            let (tx, rx) = oneshot::channel();
                            let mut dialog = rfd::AsyncFileDialog::new()
                                .add_filter("Sims 2/3 package files (.package)", &["package"]);
                            if let Some(path) = self.data.open_file_path.as_ref()
                                .and_then(|p| p.parent()){
                                dialog = dialog.set_directory(path);
                            }
                            let dialog = dialog.save_file();
                            async_execute(async move {
                                let file = dialog.await;
                                let _ = if let Some(handle) = file {
                                    tx.send(Some(handle))
                                } else {
                                    tx.send(None)
                                };
                            });
                            self.save_file_picker = Some(rx);
                        }
                    } else {
                        ui.add_enabled(false, Button::new("💾"));
                        ui.add_enabled(false, Button::new("💾✏"));
                    }
                });
            });

        let style = egui_dock::Style::from_egui(ctx.style().as_ref());
        egui_dock::DockArea::new(&mut self.dock_state)
            .style(style)
            .show(ctx, &mut self.data);

        if let Some(new_tab_index) = self.data.open_new_tab_index {
            self.data.open_new_tab_index = None;
            self.open_index_tab(new_tab_index, false, ctx);
        }
        if let Some(new_hex_tab_index) = self.data.open_new_hex_tab_index {
            self.data.open_new_hex_tab_index = None;
            self.open_index_tab(new_hex_tab_index, true, ctx);
        }

        ctx.input(|i| i.raw.dropped_files.get(0).map(|f| f.clone()))
            .map(|file| {
                if let Some(path) = file.path {
                    self.open_file(path);
                } else if let Some(bytes) = file.bytes {
                    self.open_bytes(Vec::from(&*bytes));
                }
                ctx.request_discard("load file from dropped input");
            });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        // save editor tab rc indices
        if let Some((_data, _file, rc_entries)) = &self.data.open_file {
            self.dock_state.iter_all_tabs_mut().for_each(|(_i, tab)| {
                match tab {
                    YaPeTab::File => {}
                    YaPeTab::Entry(entry) => {
                        entry.index = rc_entries.iter()
                            .enumerate()
                            .find_map(|(i, elem)| {
                                if entry.data.ptr_eq(&Rc::downgrade(elem)) {
                                    Some(i)
                                } else {
                                    None
                                }
                            });
                    }
                }
            });
        }

        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    files: Vec<PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    graphical_application_main(
        include_bytes!("../icon.png"),
        "Yet Another Package Editor",
        Box::new(|cc|
            Ok(Box::new(YaPeApp::new(cc, args)))))
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(YaPeApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
