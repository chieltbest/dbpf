#![windows_subsystem = "windows"]

mod filtered_conflict_list;

use crate::filtered_conflict_list::{ConflictTypeFilterWarning, FilteredConflictList};

use std::error::Error;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};
use eframe::{App, egui, Frame, IconData, NativeOptions, Storage};
use eframe::egui::{Color32, containers, Context, DragValue, Label, ProgressBar, RichText, Sense, Style, TextEdit, Ui, Visuals};
use egui_extras::Column;
use futures::channel::oneshot;
use rfd::FileHandle;
use tracing::{info, instrument, warn};
use tracing_subscriber::layer::SubscriberExt;
use dbpf_utils::tgi_conflicts::{find_conflicts, TGI, TGIConflict};

struct DBPFApp {
    ui_scale: f32,
    dark_mode_preference: Option<bool>,
    scan_folders: String,
    show_folders: bool,

    scan_ran_with_folders: Vec<PathBuf>,
    downloads_picker: Option<oneshot::Receiver<Option<Vec<FileHandle>>>>,

    conflict_list: FilteredConflictList,
    find_conflicts_result_stream: Option<Receiver<TGIConflict>>,
    find_conflict_progress: Arc<Mutex<Option<(PathBuf, usize, usize)>>>,
    highlighted_conflict: Option<TGIConflict>,
}

impl DBPFApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut new = Self {
            ui_scale: 1.0,
            dark_mode_preference: None,
            scan_folders: "".to_string(),
            show_folders: true,

            scan_ran_with_folders: Vec::new(),
            downloads_picker: None,

            conflict_list: FilteredConflictList::new(),
            find_conflicts_result_stream: None,
            find_conflict_progress: Mutex::new(None).into(),
            highlighted_conflict: None,
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
                new.set_dark_mode(dark_mode_preference, &cc.egui_ctx);
            }
            if let Some(downloads_folder) = storage
                .get_string("downloads_folder") {
                new.scan_folders = downloads_folder;
                new.start_scannning(&cc.egui_ctx);
            }
            if let Some(show_folders) = storage
                .get_string("show_folders")
                .and_then(|str| str.parse().ok()) {
                new.show_folders = show_folders;
            }

            for t in FilteredConflictList::filter_types() {
                if let Some(enable) = storage
                    .get_string(format!("check_{}", t.properties().abbreviation).as_str())
                    .and_then(|str| str.parse().ok()) {
                    new.conflict_list.set_check_enabled(&t, enable);
                }
            }
        }
        new
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

    fn open_downloads_picker(&mut self) {
        let mut dialog = rfd::AsyncFileDialog::new();
        let cur_dir_path = PathBuf::from(&self.scan_folders.lines().next().unwrap_or(""));
        if cur_dir_path.is_dir() {
            dialog = dialog.set_directory(cur_dir_path);
        } else if let Some(dir) = cur_dir_path.parent() {
            if dir.is_dir() {
                dialog = dialog.set_directory(dir);
            }
        }
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(|| {
            let _ = tx.send(futures::executor::block_on(dialog.pick_folders()));
        });
        self.downloads_picker = Some(rx);
    }

    #[instrument(skip(self))]
    fn start_scannning(&mut self, ctx: &Context) {
        self.conflict_list.clear();
        self.scan_ran_with_folders = self.scan_folders.lines().map(|line| PathBuf::from(line)).collect();
        self.highlighted_conflict = None;

        let (tx, rx) = mpsc::channel();
        let progress_clone = Arc::clone(&self.find_conflict_progress);
        let ctx_clone = ctx.clone();
        tokio::task::spawn(find_conflicts(
            self.scan_ran_with_folders.clone(),
            tx,
            move |path, current, total| {
                info!(path = ?path.display(), current, total, "scanning");
                let mut data = progress_clone.lock().unwrap();
                *data = if current != total {
                    Some((path, current, total))
                } else {
                    None
                };
                ctx_clone.request_repaint();
            }));
        self.find_conflicts_result_stream = Some(rx);
    }

    fn update_state(&mut self, ctx: &Context) {
        // pull in the newly found conflicts before showing them
        let mut drop_stream = false;
        if let Some(ref stream) = self.find_conflicts_result_stream {
            while match stream.try_recv() {
                Ok(conflict) => {
                    self.conflict_list.add(conflict);
                    true
                }
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Disconnected) => {
                    drop_stream = true;
                    false
                }
            } {}
        }
        if drop_stream {
            self.find_conflicts_result_stream = None;
        }

        // check for a downloads folder picker response
        if let Some(ref mut picker) = self.downloads_picker {
            match picker.try_recv() {
                Ok(None) => {}
                Ok(Some(res)) => {
                    if let Some(folders) = res {
                        self.scan_folders = folders.iter()
                            .map(|folder| folder.path().to_string_lossy().to_string())
                            .reduce(|mut full, str| {
                                full.push_str("\n");
                                full.push_str(str.as_str());
                                full
                            }).unwrap_or("".to_string());
                        self.start_scannning(ctx);
                    }
                    self.downloads_picker = None;
                }
                _ => {
                    self.downloads_picker = None;
                }
            }
        }
    }

    fn resource_menu(&mut self, ui: &mut Ui) {
        let hidden_conflicts = self.conflict_list.has_hidden_conflicts();
        ui.menu_button(format!("Resources{}", if hidden_conflicts { " ！" } else { "" }), |ui| {
            egui::Grid::new("resource grid")
                .striped(true)
                .min_col_width(0.0)
                .show(ui, |ui| {
                    for file_type in FilteredConflictList::filter_types() {
                        let mut check = self.conflict_list.get_check_enabled(&file_type);
                        ui.checkbox(&mut check, file_type.properties().abbreviation.to_string())
                            .on_hover_text(format!("Search for {} conflicts?", file_type.properties().name))
                            .changed().then(|| {
                            self.conflict_list.set_check_enabled(&file_type, check);
                        });

                        match self.conflict_list.get_type_visibility(&file_type) {
                            ConflictTypeFilterWarning::NotVisible => ui.label("！")
                                .on_hover_text("Some conflicts of this type are found but not shown"),
                            ConflictTypeFilterWarning::FoundVisible => ui.label("✔")
                                .on_hover_text("Conflicts of this type have been found, and all are shown"),
                            ConflictTypeFilterWarning::NotFound => ui.label(""),
                        };

                        ui.label(file_type.properties().name);

                        ui.end_row();
                    }
                });

            if ui.button("Reset to defaults").clicked() {
                self.conflict_list.reset_filters();
            }
        }).response
            .on_hover_text(format!("The resource types to check for{}",
                                   if hidden_conflicts {
                                       "\nSome found conflicts are not shown"
                                   } else { "" }));
    }

    fn conflict_description_string(path: &Path, tgis: &Vec<TGI>) -> String {
        let mut desc = path.to_string_lossy().to_string();
        for tgi in tgis {
            desc.push_str(format!("\n{tgi:X?}").as_str());
        }
        desc
    }

    #[instrument(skip(ui))]
    fn conflict_menu(path: &Path, tgis: &Vec<TGI>, ui: &mut Ui) {
        if ui.button("Copy name").clicked() {
            if let Some(stem) = path.file_stem().and_then(|str| str.to_str()) {
                ui.output_mut(|o| o.copied_text = stem.to_string())
            } else {
                warn!("could not get file stem");
            }
            ui.close_menu();
        }
        if ui.button("Copy name.package").clicked() {
            if let Some(name) = path.file_name().and_then(|str| str.to_str()) {
                ui.output_mut(|o| o.copied_text = name.to_string());
            } else {
                warn!("could not get filename");
            }
            ui.close_menu();
        }
        if ui.button("Copy full path").clicked() {
            ui.output_mut(|o| o.copied_text = path.to_string_lossy().to_string());
            ui.close_menu();
        }
        if ui.button("Copy full conflict data").clicked() {
            ui.output_mut(|o| o.copied_text = Self::conflict_description_string(path, tgis));
            ui.close_menu();
        }
    }

    fn strip_prefix<'a>(&self, path: &'a Path) -> Option<&'a Path> {
        self.scan_ran_with_folders.iter()
            .find_map(|folder| path.strip_prefix(folder).ok())
    }

    fn show_path_cell(&self, conflict: &TGIConflict, path: &PathBuf, ui: &mut Ui) -> bool {
        let path_same = conflict.original == conflict.new;

        let stripped_path = self.strip_prefix(path).unwrap_or(path);

        let mut text_string = if self.show_folders {
            stripped_path.to_string_lossy().to_string()
        } else {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or(stripped_path.to_string_lossy().to_string())
        };
        if path_same {
            text_string.insert_str(0, "✔ ");
        }

        let mut text = RichText::new(text_string);

        if path_same {
            text = text.color(Color32::DARK_GREEN);
        }

        let tooltip = Self::conflict_description_string(stripped_path, &conflict.tgis);

        let mut frame = containers::Frame::none();
        let selected = self.highlighted_conflict.as_ref().map(|c| conflict == c).unwrap_or(false);
        if selected {
            frame.fill = if ui.style().visuals.dark_mode {
                Color32::DARK_GRAY
            } else {
                Color32::LIGHT_GRAY
            };
        }

        let mut highlight = false;

        frame.show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add(Label::new(text)
                    .wrap(false)
                    .sense(Sense::click()))
                    .context_menu(|ui| Self::conflict_menu(path, &conflict.tgis, ui))
                    .on_hover_text_at_pointer(tooltip)
                    .clicked().then(|| {
                    highlight = true;
                });

                ui.centered_and_justified(|ui| ui.label(""));
            });
        });

        highlight
    }

    #[instrument(skip_all)]
    fn show_table(&mut self, ui: &mut Ui) {
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .column(Column::remainder().at_least(100.0).clip(true).resizable(true))
            .column(Column::remainder().at_least(100.0).clip(true))
            .max_scroll_height(f32::MAX)
            .header(30.0, |mut row| {
                row.col(|ui| { ui.heading("Original"); });
                row.col(|ui| { ui.heading("Conflict"); });
            })
            .body(|body| {
                let filtered = self.conflict_list.get_filtered();
                let mut highlight = None;
                body.rows(14.0, filtered.len(),
                          |i, mut row| {
                              let conflict = filtered[i].clone();
                              row.col(|ui| {
                                  if self.show_path_cell(
                                      &conflict,
                                      &conflict.original,
                                      ui) {
                                      highlight = Some(conflict.clone());
                                  }
                              });
                              row.col(|ui| {
                                  if self.show_path_cell(
                                      &conflict,
                                      &conflict.new,
                                      ui) {
                                      highlight = Some(conflict.clone());
                                  }
                              });
                          });
                if let Some(_) = highlight {
                    self.highlighted_conflict = highlight;
                }
            });
    }
}

impl App for DBPFApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.update_state(ctx);

        if !frame.is_web() {
            egui::gui_zoom::zoom_with_keyboard_shortcuts(ctx, frame.info().native_pixels_per_point);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
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

                    self.resource_menu(ui);

                    ui.checkbox(&mut self.show_folders, "Show paths")
                        .on_hover_text("Show what folders the packages are in?");
                });

                ui.horizontal(|ui| {
                    ui.label("Downloads: ");

                    ui.add(TextEdit::singleline(&mut self.scan_folders)
                        .id_source("scan folders")
                        .desired_width(ui.available_width() - 30.0)
                        // .desired_rows(1)
                        /*.lock_focus(false)*/)
                        .lost_focus().then(|| {
                        self.start_scannning(ctx);
                    });
                    if ui.button("🗁").clicked() {
                        self.open_downloads_picker();
                    }
                }).response.on_hover_text_at_pointer("The folder you want to scan (normally your downloads folder)");

                if let Some((ref path, progress, total)) = *self.find_conflict_progress.lock().unwrap() {
                    ui.add(ProgressBar::new(progress as f32 / total as f32)
                        .text(self.strip_prefix(path)
                            .unwrap_or(path)
                            .display().to_string()));
                }

                ui.separator();

                self.show_table(ui);
            });
        });
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        storage.set_string("ui_scale", self.ui_scale.to_string());
        if let Some(dark) = self.dark_mode_preference {
            storage.set_string("dark_mode", dark.to_string());
        }
        storage.set_string("downloads_folder", self.scan_folders.clone());
        storage.set_string("show_folders", self.show_folders.to_string());

        for t in FilteredConflictList::filter_types() {
            storage.set_string(format!("check_{}", t.properties().abbreviation).as_str(),
                               self.conflict_list.get_check_enabled(&t).to_string());
        }
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

    eframe::run_native("Yet Another Conflict Tool",
                       native_options,
                       Box::new(|cc|
                           Box::new(DBPFApp::new(cc))))?;
    Ok(())
}
