use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, TryRecvError};
use eframe::{App, egui, Error, Frame, NativeOptions, Storage};
use eframe::egui::{Color32, Context, Label, RichText, Slider, Style, TextEdit, Visuals};
use egui_extras::Column;
use futures::channel::oneshot;
use rfd::FileHandle;
use dbpf_utils::tgi_conflicts::{find_conflicts, TGI, TGIConflict};

struct DBPFApp {
    ui_scale: f32,
    dark_mode_preference: Option<bool>,
    downloads_dir: String,

    scan_ran_with_dir: PathBuf,
    downloads_picker: Option<oneshot::Receiver<Option<FileHandle>>>,

    found_conflicts: Vec<TGIConflict>,
    found_conflicts_stream: Option<Receiver<TGIConflict>>,
}

impl DBPFApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut new = Self {
            ui_scale: 1.0,
            dark_mode_preference: None,
            downloads_dir: "".to_string(),

            scan_ran_with_dir: PathBuf::new(),
            downloads_picker: None,
            found_conflicts: Vec::new(),
            found_conflicts_stream: None,
        };
        if let Some(storage) = cc.storage {
            if let Some(ui_scale) = storage
                .get_string("ui_scale")
                .and_then(|str| str.parse().ok()) {
                new.ui_scale = ui_scale;
                cc.egui_ctx.set_pixels_per_point(ui_scale);
            }
            if let Some(dark_mode_preference) = storage
                .get_string("dark_mode")
                .and_then(|str| str.parse().ok()) {
                new.dark_mode_preference = Some(dark_mode_preference);
                cc.egui_ctx.set_style(Style {
                    visuals: if dark_mode_preference {
                        Visuals::dark()
                    } else {
                        Visuals::light()
                    },
                    ..Default::default()
                });
            }
            if let Some(downloads_dir) = storage
                .get_string("downloads_dir") {
                new.downloads_dir = downloads_dir;
                new.start_scannning();
            }
        }
        new
    }
}

impl DBPFApp {
    fn start_scannning(&mut self) {
        self.found_conflicts = Vec::new();
        self.scan_ran_with_dir = PathBuf::from(&self.downloads_dir);
        let (tx, rx) = mpsc::channel();
        tokio::task::spawn(find_conflicts(self.scan_ran_with_dir.clone(), tx));
        self.found_conflicts_stream = Some(rx);
    }
}

impl App for DBPFApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let dark_mode = self.dark_mode_preference.unwrap_or(ui.style().visuals.dark_mode);
                    ui.button(if dark_mode { "☀" } else { "🌙" })
                        .on_hover_text(format!("Switch to {} mode",
                                               if dark_mode { "light" } else { "dark" }))
                        .clicked().then(|| {
                        self.dark_mode_preference = Some(!dark_mode);
                        ctx.set_style(Style {
                            visuals: if !dark_mode {
                                Visuals::dark()
                            } else {
                                Visuals::light()
                            },
                            ..Default::default()
                        })
                    });
                    ui.spacing_mut().slider_width = ui.available_width() - 100.0;
                    ui.add(Slider::new(&mut self.ui_scale, 0.5..=5.0)
                        .text("UI Scale").fixed_decimals(1))
                        .changed().then(|| {
                        ctx.set_pixels_per_point(self.ui_scale);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Downloads: ");
                    ui.add_sized([ui.available_width() - 30.0, 20.0],
                                 TextEdit::singleline(&mut self.downloads_dir))
                        .lost_focus().then(|| {
                        self.start_scannning();
                    });
                    if ui.button("🗁").clicked() {
                        let mut dialog = rfd::AsyncFileDialog::new();
                        let cur_dir_path = PathBuf::from(&self.downloads_dir);
                        if cur_dir_path.is_dir() {
                            dialog = dialog.set_directory(cur_dir_path);
                        } else if let Some(dir) = cur_dir_path.parent() {
                            if dir.is_dir() {
                                dialog = dialog.set_directory(dir);
                            }
                        }
                        let (tx, rx) = oneshot::channel();
                        std::thread::spawn(|| {
                            let _ = tx.send(futures::executor::block_on(dialog.pick_folder()));
                        });
                        self.downloads_picker = Some(rx);
                    }
                    if let Some(ref mut picker) = self.downloads_picker {
                        match picker.try_recv() {
                            Ok(None) => {}
                            Ok(Some(res)) => {
                                if let Some(folder) = res {
                                    self.downloads_dir = folder.path().to_string_lossy().to_string();
                                    self.start_scannning();
                                }
                                self.downloads_picker = None;
                            }
                            _ => {
                                self.downloads_picker = None;
                            }
                        }
                    }
                }).response.on_hover_text_at_pointer("Your downloads folder");

                if let Some(_) = self.found_conflicts_stream {
                    // scan is in progress
                    ui.spinner();
                }

                ui.separator();

                egui_extras::TableBuilder::new(ui)
                    .stick_to_bottom(true)
                    .striped(true)
                    .column(Column::remainder().at_least(100.0).clip(true).resizable(true))
                    .column(Column::remainder().at_least(100.0).clip(true))
                    .max_scroll_height(f32::MAX)
                    .header(30.0, |mut row| {
                        row.col(|ui| { ui.heading("original"); });
                        row.col(|ui| { ui.heading("conflict"); });
                    })
                    .body(|body| {
                        // pull in the newly found conflicts before showing them
                        let mut drop_stream = false;
                        if let Some(ref stream) = self.found_conflicts_stream {
                            while match stream.try_recv() {
                                Ok(conflict) => {
                                    self.found_conflicts.push(conflict);
                                    true
                                }
                                Err(TryRecvError::Empty) => false,
                                Err(TryRecvError::Disconnected) => {
                                    drop_stream = true;
                                    false
                                }
                            } {}
                            ctx.request_repaint();
                        }
                        if drop_stream {
                            self.found_conflicts_stream = None;
                        }

                        let show_path_cell = |path: &PathBuf, tgis: &Vec<TGI>, path_same, ui: &mut egui::Ui| {
                            let stripped_path = path
                                .strip_prefix(&self.scan_ran_with_dir)
                                .unwrap_or(Path::new(""))
                                .to_string_lossy().to_string();

                            let mut text = RichText::new(path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .map(|str| if path_same {
                                    let mut str = str.to_string();
                                    str.insert_str(0, "✔ ");
                                    str
                                } else {
                                    str.to_string()
                                })
                                .unwrap_or(stripped_path.clone()));

                            if path_same {
                                text = if ui.style().visuals.dark_mode {
                                    text.color(Color32::DARK_GREEN)
                                } else {
                                    text.background_color(Color32::LIGHT_GREEN)
                                };
                            }

                            let mut tooltip = stripped_path;
                            tooltip.push_str(":");
                            for tgi in tgis {
                                tooltip.push_str(format!("\n{tgi:X?}").as_str());
                            }

                            ui.add(Label::new(text)
                                .wrap(false))
                                .on_hover_text_at_pointer(tooltip);
                        };

                        body.rows(14.0, self.found_conflicts.len(),
                                  |i, mut row| {
                                      let conflict = &self.found_conflicts[i];
                                      let is_internal = conflict.original == conflict.new;
                                      row.col(|ui| {
                                          let orig_path = &conflict.original;
                                          show_path_cell(orig_path, &conflict.tgis, is_internal, ui);
                                      });
                                      row.col(|ui| {
                                          let new_path = &conflict.new;
                                          show_path_cell(new_path, &conflict.tgis, is_internal, ui);
                                      });
                                  })
                    });
            });
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        storage.set_string("ui_scale", self.ui_scale.to_string());
        if let Some(dark) = self.dark_mode_preference {
            storage.set_string("dark_mode", dark.to_string());
        }
        storage.set_string("downloads_dir", self.downloads_dir.clone());
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let native_options = NativeOptions {
        drag_and_drop_support: true,
        resizable: true,
        ..Default::default()
    };

    eframe::run_native("DBPF App",
                       native_options,
                       Box::new(|cc|
                           Box::new(DBPFApp::new(cc))))
}
