use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use binrw::BinResult;
use std::io::{Cursor, Read, Seek};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use eframe::{App, egui, Frame, IconData, NativeOptions, Storage};
use eframe::egui::{Color32, Context, DragValue, Id, Rect, Response, ScrollArea, Sense, Style, Ui, Visuals, WidgetText};
use egui_dock::{Node, TabViewer, Tree};
use egui_extras::Column;
use egui_memory_editor::MemoryEditor;
use egui_memory_editor::option_data::MemoryEditorOptions;
use serde::{Deserialize, Serialize};
use tracing_subscriber::layer::SubscriberExt;
use dbpf::{CompressionType, DBPFFile, IndexEntry};
use dbpf::internal_file::CompressionError;

use crate::editor::{DecodedFileEditorState, Editor, editor_supported};

mod editor;

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

    // used for (de)serialising
    index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
enum YaPeTab {
    File,
    Entry(EntryEditorTab),
}

#[derive(Serialize, Deserialize)]
struct YaPeAppData {
    memory_editor_options: MemoryEditorOptions,

    open_file_path: Option<PathBuf>,

    #[serde(skip)]
    open_file: Option<(Cursor<Vec<u8>>, BinResult<DBPFFile>, Vec<Rc<RefCell<IndexEntry>>>)>,

    #[serde(skip)]
    open_new_tab_index: Option<usize>,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct YaPeApp {
    ui_scale: f32,
    dark_mode_preference: Option<bool>,

    tab_tree: Tree<YaPeTab>,

    data: YaPeAppData,
}

impl Default for YaPeApp {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            dark_mode_preference: None,

            tab_tree: Tree::new(vec![YaPeTab::File]),

            data: YaPeAppData {
                memory_editor_options: MemoryEditorOptions::default(),

                open_file_path: None,

                open_file: None,

                open_new_tab_index: None,
            },
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
                        ui.centered_and_justified(|ui|
                            ScrollArea::vertical().show(ui, |ui| {
                                if ui.code_editor(&mut str).changed() {
                                    data.data = str.into_bytes();
                                }
                            })
                        );
                    } else {
                        editor.draw_editor_contents(
                            ui,
                            data,
                            |mem, addr| Some(mem.data[addr]),
                            |mem, addr, byte| mem.data[addr] = byte,
                        );
                    }
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

        match &mut self.open_file {
            None => {}
            Some((_, Err(err), _entries)) => {
                ui.colored_label(Color32::RED, err.to_string());
            }
            Some((_, Ok(_file), entries)) => {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
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
                    .header(30.0, |mut row| {
                        row.col(|ui| { ui.label("Type"); });
                        row.col(|ui| { ui.label("Group"); });
                        row.col(|ui| { ui.label("Instance"); });
                        row.col(|ui| { ui.label("Compression"); });
                    })
                    .body(|mut body| {
                        let button_height = body.ui_mut().style().spacing.interact_size.y;
                        body.rows(button_height, entries.len(),
                                  |i, mut row| {
                                      let mut sense_fun = |ui: &mut Ui, res: Response| {
                                          if ui.interact(
                                              Rect::everything_right_of(res.rect.right()),
                                              Id::from(format!("row_interact_{i}")),
                                              Sense::click(),
                                          ).clicked() {
                                              open_index = Some(i);
                                          }
                                      };

                                      let mut entry = entries[i].borrow_mut();
                                      row.col(|ui| {
                                          let t = entry.type_id;
                                          let res = ui.horizontal_centered(|ui|
                                              ui.label(t.properties().map_or_else(
                                                  || format!("{:08X}", t.code()),
                                                  |prop| prop.abbreviation.to_string())));
                                          sense_fun(ui, res.inner);
                                      });
                                      row.col(|ui| {
                                          let res = ui.add(DragValue::new(&mut entry.group_id)
                                              .hexadecimal(8, false, true));
                                          sense_fun(ui, res);
                                      });
                                      row.col(|ui| {
                                          let res = ui.add(DragValue::new(&mut entry.instance_id)
                                              .hexadecimal(8, false, true));
                                          sense_fun(ui, res);
                                      });
                                      row.col(|ui| {
                                          let res = egui::ComboBox::from_id_source(
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
                                          sense_fun(ui, res.response);
                                      });
                                  });
                    });
            }
        }

        self.open_new_tab_index = open_index;
    }
}

impl TabViewer for YaPeAppData {
    type Tab = YaPeTab;

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

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        match tab {
            YaPeTab::File => "Index".into(),
            YaPeTab::Entry(entry) => {
                if let Some(index) = entry.data.upgrade() {
                    index.borrow().type_id.full_name().into()
                } else {
                    "Unknown".into()
                }
            }
        }
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            YaPeTab::File => false,
            YaPeTab::Entry(entry) => entry.data.strong_count() == 0,
        }
    }
}

impl YaPeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut new: YaPeApp = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            cc.egui_ctx.set_pixels_per_point(new.ui_scale);

            if let Some(path) = new.data.open_file_path.clone() {
                new.open_file(std::fs::read(path).unwrap());

                if let Some((data, _file, rc_entries)) = &mut new.data.open_file {
                    new.tab_tree.iter_mut().for_each(|elem| {
                        match elem {
                            Node::Leaf { tabs, .. } => {
                                tabs.iter_mut().for_each(|elem| {
                                    match elem {
                                        YaPeTab::File => {}
                                        YaPeTab::Entry(entry) => {
                                            if let Some(i) = entry.index {
                                                *entry = Self::open_index(data, rc_entries, i);
                                            }
                                        }
                                    }
                                });
                            }
                            _ => {}
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

    fn open_file(&mut self, bytes: Vec<u8>) {
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
            Err(_) => vec![],
        };

        self.data.open_file = Some((cursor, parsed, rc_index));


        let mut file_index_tab_found = false;
        self.tab_tree.tabs().for_each(|tab| {
            match tab {
                YaPeTab::File => {
                    file_index_tab_found = true;
                }
                YaPeTab::Entry(_) => {}
            }
        });
        if !file_index_tab_found {
            self.tab_tree.push_to_first_leaf(YaPeTab::File);
        }
    }

    #[must_use]
    fn open_index<R: Read + Seek>(reader: &mut R, rc_entries: &Vec<Rc<RefCell<IndexEntry>>>, index: usize) -> EntryEditorTab {
        let index_entry = &rc_entries[index];
        let mut entry_ref = index_entry.borrow_mut();
        let file_type = entry_ref.type_id;
        let res = entry_ref.data(reader)
            .map_err(|err| CompressionError::BinResult(err))
            .and_then(|entry| {
                if editor_supported(file_type) {
                    let decoded = entry.decoded()?;
                    Ok(EditorType::DecodedEditor(decoded.unwrap().new_editor()))
                } else {
                    let decompressed = entry.decompressed()?;
                    Ok(EditorType::HexEditor(
                        MemoryEditor::new().with_address_range(
                            "", 0..decompressed.data.len())))
                }
            });

        match res {
            Err(err) => {
                EntryEditorTab {
                    state: EditorType::Error(err),
                    data: Rc::downgrade(index_entry),
                    index: None,
                }
            }
            Ok(editor_state) => {
                EntryEditorTab {
                    state: editor_state,
                    data: Rc::downgrade(index_entry),
                    index: None,
                }
            }
        }
    }

    fn open_index_tab(&mut self, index: usize) {
        if let Some((cur, _file, rc_entries)) = &mut self.data.open_file {
            let tab = Self::open_index(cur, rc_entries, index);
            self.tab_tree.push_to_focused_leaf(YaPeTab::Entry(tab));
        }
    }
}

impl App for YaPeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.input(|i| i.raw.dropped_files.get(0).map(|f| f.clone()))
            .map(|file| {
                if let Some(path) = file.path {
                    self.data.open_file_path = Some(path.clone());
                    self.open_file(std::fs::read(path).unwrap());
                } else if let Some(bytes) = file.bytes {
                    self.open_file(Vec::from(&*bytes));
                }
            });

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
                });
            });

        let style = egui_dock::Style::from_egui(ctx.style().as_ref());
        egui_dock::DockArea::new(&mut self.tab_tree)
            .style(style)
            .show(ctx, &mut self.data);

        if let Some(new_tab_index) = self.data.open_new_tab_index {
            self.data.open_new_tab_index = None;
            self.open_index_tab(new_tab_index);
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        // save editor tab rc indices
        if let Some((_data, _file, rc_entries)) = &self.data.open_file {
            self.tab_tree.iter_mut().for_each(|node| {
                match node {
                    Node::Empty => {}
                    Node::Leaf { tabs, .. } => {
                        tabs.iter_mut().for_each(|tab| {
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
                        })
                    }
                    Node::Vertical { .. } => {}
                    Node::Horizontal { .. } => {}
                }
            });
        }

        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing::subscriber::set_global_default(tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
    ).expect("set up the subscriber");

    let icon = include_bytes!("../../../res/yact.png");
    let image = image::io::Reader::new(Cursor::new(icon))
        .with_guessed_format()?.decode()?;
    let buf = Vec::from(image.as_bytes());

    let native_options = NativeOptions {
        icon_data: Some(IconData {
            width: image.width(),
            height: image.height(),
            rgba: buf,
        }),
        drag_and_drop_support: true,
        resizable: true,
        ..Default::default()
    };

    eframe::run_native("Yet Another Package Editor",
                       native_options,
                       Box::new(|cc|
                           Box::new(YaPeApp::new(cc))))?;
    Ok(())
}
