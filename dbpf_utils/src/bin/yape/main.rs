use std::error::Error;
use binrw::BinResult;
use std::io::Cursor;
use eframe::{App, egui, Frame, IconData, NativeOptions, Storage};
use eframe::egui::{Align, Color32, Context, DragValue, Id, Layout, Rect, Response, ScrollArea, Sense, Style, Ui, Visuals};
use egui_extras::Column;
use egui_memory_editor::MemoryEditor;
use egui_memory_editor::option_data::MemoryEditorOptions;
use serde::{Deserialize, Serialize};
use tracing_subscriber::layer::SubscriberExt;
use dbpf::{CompressionType, DBPFFile};
use dbpf::internal_file::CompressionError;

use editor::Editor;
use crate::editor::editor_supported;

mod editor;

enum EditorType {
    HexEditor(MemoryEditor),
    DecodedEditor,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct YaPeApp {
    ui_scale: f32,
    dark_mode_preference: Option<bool>,
    memory_editor_options: MemoryEditorOptions,

    #[serde(skip)]
    open_file: Option<(Cursor<Vec<u8>>, BinResult<DBPFFile>)>,
    #[serde(skip)]
    open_entry: Option<Result<(EditorType, usize), CompressionError>>,
}

impl Default for YaPeApp {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            dark_mode_preference: None,
            memory_editor_options: MemoryEditorOptions::default(),

            open_file: None,
            open_entry: None,
        }
    }
}

impl YaPeApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let new: YaPeApp = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            cc.egui_ctx.set_pixels_per_point(new.ui_scale);
            return new;
        }

        Self::default()
    }
}

impl YaPeApp {
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
        let parsed = DBPFFile::read(&mut cursor);
        self.open_file = Some((cursor, parsed));
        self.open_entry = None;
    }

    fn open_index(&mut self, index: usize) {
        if let Some((cur, Ok(file))) = &mut self.open_file {
            let file_type = file.index[index].type_id;
            let res = file.index[index].data(cur)
                .map_err(|err| CompressionError::BinResult(err));

            match res.and_then(|entry| {
                if editor_supported(file_type) {
                    entry.decoded()?;
                    Ok(Some(Ok((EditorType::DecodedEditor, index))))
                } else {
                    let decompressed = entry.decompressed()?;
                    Ok(Some(Ok((
                        EditorType::HexEditor(
                            MemoryEditor::new().with_address_range(
                                "",
                                0..decompressed.data.len())),
                        index))))
                }
            }) {
                Err(err) => {
                    self.open_entry = Some(Err(err))
                }
                Ok(open) => {
                    self.open_entry = open;
                }
            }
        }
    }

    fn show_index(&mut self, ui: &mut Ui) {
        let mut open_index = None;

        match &mut self.open_file {
            None => {}
            Some((_, Err(err))) => {
                ui.colored_label(Color32::RED, err.to_string());
            }
            Some((_, Ok(file))) => {
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
                        body.rows(button_height, file.index.len(),
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

                                      let entry = &mut file.index[i];
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

        if let Some(i) = open_index {
            self.open_index(i);
        }
    }

    fn show_editor(&mut self, ui: &mut Ui) {
        if let Some((reader, Ok(file))) = &mut self.open_file {
            match &mut self.open_entry {
                None => {}
                Some(Err(err)) => {
                    ui.colored_label(Color32::RED, format!("{err:?}"));
                }
                Some(Ok((editor, i))) => {
                    match editor {
                        EditorType::HexEditor(editor) => {
                            let data = file.index[*i].data(reader).unwrap().decompressed().unwrap();
                            if let Ok(mut str) = String::from_utf8(data.data.clone()) {
                                ui.centered_and_justified(|ui|
                                    ScrollArea::vertical().show(ui, |ui| {
                                        if ui.code_editor(&mut str).changed() {
                                            data.data = str.into_bytes();
                                        }
                                    })
                                );
                            } else {
                                // this method of persisting config is ugly but robust
                                editor.options = self.memory_editor_options.clone();
                                // the editor can change some internal config data
                                editor.draw_editor_contents(
                                    ui,
                                    data,
                                    |mem, addr| Some(mem.data[addr]),
                                    |mem, addr, byte| mem.data[addr] = byte,
                                );
                                // then copy it back to the main config
                                self.memory_editor_options = editor.options.clone();
                            }
                        }
                        EditorType::DecodedEditor => {
                            let decoded = file.index[*i].data(reader).unwrap().decoded().unwrap().unwrap();
                            decoded.show_editor(ui);
                        }
                    }
                }
            }
        }
    }
}

impl App for YaPeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.input(|i| i.raw.dropped_files.get(0).map(|f| f.clone()))
            .map(|file| {
                if let Some(bytes) = file.bytes {
                    self.open_file(Vec::from(&*bytes));
                } else if let Some(path) = file.path {
                    self.open_file(std::fs::read(path).unwrap());
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

        egui::SidePanel::right("data_display")
            .resizable(true)
            .max_width(ctx.available_rect().width() - 500.0)
            .show(ctx, |ui| {
                self.show_editor(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_index(ui);
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
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
