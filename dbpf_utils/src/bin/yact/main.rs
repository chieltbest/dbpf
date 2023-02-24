#![windows_subsystem = "windows"]

mod filtered_conflict_list;

use crate::filtered_conflict_list::{ConflictTypeFilterWarning, FilteredConflictList};

use std::error::Error;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, TryRecvError};
use eframe::{App, egui, Frame, IconData, NativeOptions, Storage};
use eframe::egui::{Color32, containers, Context, DragValue, Label, RichText, Sense, Style, TextEdit, Ui, Visuals};
use egui_extras::Column;
use futures::channel::oneshot;
use rfd::FileHandle;
use tracing::{instrument, warn};
use dbpf_utils::tgi_conflicts::{find_conflicts, TGI, TGIConflict};

struct DBPFApp {
    ui_scale: f32,
    dark_mode_preference: Option<bool>,
    downloads_folder: String,
    show_folders: bool,

    scan_ran_with_folder: PathBuf,
    downloads_picker: Option<oneshot::Receiver<Option<FileHandle>>>,

    conflict_list: FilteredConflictList,
    found_conflicts_stream: Option<Receiver<TGIConflict>>,
    highlighted_conflict: Option<TGIConflict>,
}

impl DBPFApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut new = Self {
            ui_scale: 1.0,
            dark_mode_preference: None,
            downloads_folder: "".to_string(),
            show_folders: true,

            scan_ran_with_folder: PathBuf::new(),
            downloads_picker: None,

            conflict_list: FilteredConflictList::new(),
            found_conflicts_stream: None,
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
                new.downloads_folder = downloads_folder;
                new.start_scannning();
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
        let cur_dir_path = PathBuf::from(&self.downloads_folder);
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

    fn start_scannning(&mut self) {
        self.conflict_list.clear();

        self.scan_ran_with_folder = PathBuf::from(&self.downloads_folder);
        self.highlighted_conflict = None;

        let (tx, rx) = mpsc::channel();
        tokio::task::spawn(find_conflicts(self.scan_ran_with_folder.clone(), tx));
        self.found_conflicts_stream = Some(rx);
    }

    fn update_state(&mut self, ctx: &Context) {
        // pull in the newly found conflicts before showing them
        let mut drop_stream = false;
        if let Some(ref stream) = self.found_conflicts_stream {
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
            ctx.request_repaint();
        }
        if drop_stream {
            self.found_conflicts_stream = None;
        }

        // check for a downloads folder picker response
        if let Some(ref mut picker) = self.downloads_picker {
            match picker.try_recv() {
                Ok(None) => {}
                Ok(Some(res)) => {
                    if let Some(folder) = res {
                        self.downloads_folder = folder.path().to_string_lossy().to_string();
                        self.start_scannning();
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
            egui::Grid::new("resource grid").show(ui, |ui| {
                for file_type in FilteredConflictList::filter_types() {
                    let mut name = file_type.properties().abbreviation.to_string();
                    match self.conflict_list.get_type_visibility(&file_type) {
                        ConflictTypeFilterWarning::NotVisible => name.push_str(" ！"),
                        ConflictTypeFilterWarning::FoundVisible => name.push_str(" ℹ"),
                        ConflictTypeFilterWarning::NotFound => {}
                    }
                    let mut check = self.conflict_list.get_check_enabled(&file_type);
                    let res = ui.checkbox(&mut check, name)
                        .on_hover_text(format!("search for {} conflicts?", file_type.properties().name));
                    if res.changed() {
                        self.conflict_list.set_check_enabled(&file_type, check);
                    }

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

    #[instrument(skip_all)]
    fn show_table(&mut self, ui: &mut Ui) {
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .column(Column::remainder().at_least(100.0).clip(true).resizable(true))
            .column(Column::remainder().at_least(100.0).clip(true))
            .max_scroll_height(f32::MAX)
            .header(30.0, |mut row| {
                row.col(|ui| { ui.heading("original"); });
                row.col(|ui| { ui.heading("conflict"); });
            })
            .body(|body| {
                let mut show_path_cell = |conflict: &TGIConflict, path: &PathBuf, ui: &mut Ui| {
                    let path_same = conflict.original == conflict.new;

                    let stripped_path = path
                        .strip_prefix(&self.scan_ran_with_folder)
                        .unwrap_or(Path::new(""));

                    let mut text_string = if self.show_folders {
                        stripped_path.to_string_lossy().to_string()
                    } else {
                        path
                            .file_name()
                            .and_then(|name| name.to_str().map(|str| str.to_string()))
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
                    frame.show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.add(Label::new(text)
                                    .wrap(false)
                                    .sense(Sense::click()))
                                .context_menu(|ui| Self::conflict_menu(path, &conflict.tgis, ui))
                                .on_hover_text_at_pointer(tooltip)
                                .clicked().then(|| {
                                self.highlighted_conflict = Some(conflict.clone());
                            });

                            ui.centered_and_justified(|ui| ui.label(""));
                        });
                    });
                };

                let filtered = self.conflict_list.get_filtered();
                body.rows(14.0, filtered.len(),
                          |i, mut row| {
                              let conflict = &filtered[i];
                              row.col(|ui| {
                                  show_path_cell(
                                      conflict,
                                      &conflict.original,
                                      ui);
                              });
                              row.col(|ui| {
                                  let new_path = &conflict.new;
                                  show_path_cell(
                                      conflict,
                                      new_path,
                                      ui);
                              });
                          })
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

                    if let Some(_) = self.found_conflicts_stream {
                        // scan is in progress
                        ui.spinner()
                            .on_hover_text(format!("A scan of the folder {} is currently running",
                                                   self.scan_ran_with_folder.display()));
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Downloads: ");
                    ui.add_sized([ui.available_width() - 30.0, 20.0],
                                 TextEdit::singleline(&mut self.downloads_folder))
                        .lost_focus().then(|| {
                        self.start_scannning();
                    });
                    if ui.button("🗁").clicked() {
                        self.open_downloads_picker();
                    }
                }).response.on_hover_text_at_pointer("The folder you want to scan (normally your downloads folder)");

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
        storage.set_string("downloads_folder", self.downloads_folder.clone());
        storage.set_string("show_folders", self.show_folders.to_string());

        for t in FilteredConflictList::filter_types() {
            storage.set_string(format!("check_{}", t.properties().abbreviation).as_str(),
                               self.conflict_list.get_check_enabled(&t).to_string());
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
