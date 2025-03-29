#![windows_subsystem = "windows"]

// TODO integrated texture viewer
// TODO texture conversion
// TODO sort?
// TODO memory problems

mod filtered_texture_list;
mod texture_finder;
mod ui_image_cache;

use std::error::Error;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};
use eframe::{App, egui, Frame, NativeOptions, Storage};
use eframe::egui::{Color32, containers, Context, DragValue, IconData, Image, Label, ProgressBar, RichText, Sense, Style, TextEdit, Ui, ViewportBuilder, Visuals, Window};
use eframe::egui::style::Interaction;
use eframe::epaint::Vec2;
use egui_extras::Column;
use futures::channel::oneshot;
use rfd::FileHandle;
use tracing::{info, instrument, warn};
use tracing_subscriber::layer::SubscriberExt;
use crate::filtered_texture_list::FilteredTextureList;
use crate::texture_finder::{find_textures, FoundTexture};
use crate::ui_image_cache::ImageCache;

const IMAGE_CACHE_N: usize = 512;
const IMAGE_MAX_SIZE: f32 = 300.0;

struct DBPFApp {
    ui_scale: f32,
    dark_mode_preference: Option<bool>,
    show_folders: bool,
    open_known_texture_gui: bool,
    scan_folders: String,

    scan_ran_with_folders: Vec<PathBuf>,
    downloads_picker: Option<oneshot::Receiver<Option<Vec<FileHandle>>>>,

    texture_list: FilteredTextureList,
    find_textures_result_stream: Option<Receiver<FoundTexture>>,
    find_textures_progress: Arc<Mutex<Option<(PathBuf, usize, usize)>>>,
    highlighted_texture: Option<FoundTexture>,
    ui_image_cache: ImageCache,
}

impl DBPFApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut new = Self {
            ui_scale: 1.0,
            dark_mode_preference: None,
            show_folders: true,
            open_known_texture_gui: false,
            scan_folders: "".to_string(),

            scan_ran_with_folders: Vec::new(),
            downloads_picker: None,

            texture_list: FilteredTextureList::new(&cc.storage),
            find_textures_result_stream: None,
            find_textures_progress: Mutex::new(None).into(),
            highlighted_texture: None,
            ui_image_cache: ImageCache::new(NonZeroUsize::new(IMAGE_CACHE_N).unwrap()),
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
            if let Some(open_gui) = storage
                .get_string("open_known_texture_gui")
                .and_then(|str| str.parse().ok()) {
                new.open_known_texture_gui = open_gui;
            }
            if let Some(show_folders) = storage
                .get_string("show_folders")
                .and_then(|str| str.parse().ok()) {
                new.show_folders = show_folders;
            }
            if let Some(downloads_folder) = storage
                .get_string("downloads_folder") {
                new.scan_folders = downloads_folder;
                new.start_scannning(&cc.egui_ctx);
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
        self.texture_list.clear();
        self.scan_ran_with_folders = self.scan_folders.lines().map(|line| PathBuf::from(line)).collect();
        self.highlighted_texture = None;

        let (tx, rx) = mpsc::channel();
        let progress_clone = Arc::clone(&self.find_textures_progress);
        let ctx_clone = ctx.clone();
        tokio::task::spawn(find_textures(
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
        self.find_textures_result_stream = Some(rx);
    }

    fn update_state(&mut self, ctx: &Context) {
        // pull in the newly found textures before showing them
        let mut drop_stream = false;
        if let Some(ref stream) = self.find_textures_result_stream {
            while match stream.try_recv() {
                Ok(texture) => {
                    self.texture_list.add(texture);
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
            self.find_textures_result_stream = None;
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

    fn texture_description_string(path: &Path, texture: &FoundTexture) -> String {
        format!("{}\n\
        Group: {:X}\n\
        Instance: {:X}\n\
        Width: {}\n\
        Height: {}\n\
        Memory size (bytes): {}\n\
        Format: {:?}\n\
        Mipmap levels: {}",
                path.to_string_lossy(),
                texture.id.tgi.group_id,
                texture.id.tgi.instance_id,
                texture.width,
                texture.height,
                texture.memory_size,
                texture.format,
                texture.mip_levels)
    }

    fn known_texture_menu(&mut self, ctx: &Context, ui: &mut Ui) {
        Window::new("Known textures")
            .resizable(true)
            .hscroll(true)
            .open(&mut self.open_known_texture_gui)
            .show(ctx, |ui| {
                let known_textures = self.texture_list.get_known();
                if known_textures.len() > 0 {
                    let mut remove = None;

                    let available_width = ui.available_width();
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::initial(100.0)
                            .at_least(100.0)
                            .at_most(available_width - 10.0)
                            .clip(true)
                            .resizable(true))
                        .columns(Column::initial(100.0)
                                     .clip(true)
                                     .resizable(true), 1)
                        .column(Column::remainder().clip(true))
                        .max_scroll_height(f32::MAX)
                        .header(30.0, |mut row| {
                            row.col(|ui| { ui.heading("Path"); });
                            // row.col(|ui| { ui.heading("Type"); });
                            row.col(|ui| { ui.heading("Group"); });
                            row.col(|ui| { ui.heading("Instance"); });
                        })
                        .body(|body| {
                            body.rows(14.0, known_textures.len(),
                                      |mut row| {
                                          let i = row.index();

                                          let mut path_label_fn = |ui: &mut Ui, path| {
                                              ui.add(Label::new(Self::strip_prefix(
                                                  &self.scan_ran_with_folders, path)
                                                  .unwrap_or(path).to_string_lossy())
                                                  .sense(Sense::click()))
                                                  .context_menu(|ui| {
                                                      ui.button("Forget known texture")
                                                          .clicked().then(|| {
                                                          remove = Some(i);
                                                          ui.close_menu();
                                                      });
                                                  });
                                          };

                                          row.col(|ui| {
                                              path_label_fn(ui, &known_textures[i].path);
                                          });
                                          // row.col(|ui| {
                                          //     ui.label(format!("{:#?}", &known_textures[i].tgi.type_id));
                                          // });
                                          row.col(|ui| {
                                              ui.label(format!("{:X?}", &known_textures[i].tgi.group_id));
                                          });
                                          row.col(|ui| {
                                              ui.label(format!("{:X?}", &known_textures[i].tgi.instance_id));
                                          });
                                      });
                        });

                    if let Some(i) = remove {
                        self.texture_list.remove_known(i);
                    }
                } else {
                    ui.label("No known textures found");
                }
            });

        ui.button("Known textures")
            .clicked().then(|| {
            self.open_known_texture_gui = !self.open_known_texture_gui;
        });
    }

    #[instrument(skip(self, ui))]
    fn texture_menu(&mut self, path: &Path, texture: &FoundTexture, ui: &mut Ui) {
        ui.button("Add to known textures")
            .clicked().then(|| {
            self.texture_list.add_known(texture.id.clone());
            ui.close_menu();
        });

        ui.button("Copy name")
            .clicked().then(|| {
            if let Some(stem) = path.file_stem().and_then(|str| str.to_str()) {
                ui.output_mut(|o| o.copied_text = stem.to_string())
            } else {
                warn!("could not get file stem");
            }
            ui.close_menu();
        });
        ui.button("Copy name.package")
            .clicked().then(|| {
            if let Some(name) = path.file_name().and_then(|str| str.to_str()) {
                ui.output_mut(|o| o.copied_text = name.to_string());
            } else {
                warn!("could not get filename");
            }
            ui.close_menu();
        });
        ui.button("Copy full path")
            .clicked().then(|| {
            ui.output_mut(|o| o.copied_text = path.to_string_lossy().to_string());
            ui.close_menu();
        });
        if let Some(parent) = path.parent() {
            ui.button("Copy directory")
                .clicked().then(|| {
                ui.output_mut(|o| o.copied_text = parent.to_string_lossy().to_string());
                ui.close_menu();
            });
        }
    }

    fn strip_prefix<'a>(scan_folders: &Vec<PathBuf>, path: &'a Path) -> Option<&'a Path> {
        scan_folders.iter().find_map(|folder| path.strip_prefix(folder).ok())
    }

    fn show_path_cell(&mut self, texture: &FoundTexture, path: &PathBuf, ui: &mut Ui) -> bool {
        let stripped_path = Self::strip_prefix(&self.scan_ran_with_folders, path).unwrap_or(path);

        let text_string = if self.show_folders {
            stripped_path.to_string_lossy().to_string()
        } else {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or(stripped_path.to_string_lossy().to_string())
        };

        let mut text = RichText::new(text_string);

        if self.texture_list.is_known(texture) {
            text = text.color(if ui.style().visuals.dark_mode {
                Color32::DARK_GRAY
            } else {
                Color32::GRAY
            });
        }

        let tooltip = Self::texture_description_string(stripped_path, &texture);

        let mut frame = containers::Frame::none();
        let selected = self.highlighted_texture.as_ref().map(|c| texture == c).unwrap_or(false);
        if selected {
            frame.fill = if ui.style().visuals.dark_mode {
                Color32::from_gray(16)
            } else {
                Color32::LIGHT_GRAY
            };
        }

        let mut highlight = false;

        let res = frame.show(ui, |ui| {
            let res = ui.horizontal_centered(|ui| {
                ui.style_mut().interaction.selectable_labels = false;
                let lbl = ui.add(Label::new(text)
                    .sense(Sense::click()));
                lbl.context_menu(|ui| self.texture_menu(path, texture, ui));
                lbl.clicked().then(|| {
                    highlight = true;
                });
                lbl.double_clicked().then(|| {
                    open::that_detached(path).unwrap();
                });

                let res = ui.centered_and_justified(|ui| ui.label(""));
                let res = res.response | res.inner;
                lbl | res
            });

            res.response | res.inner
        });
        let res = res.response | res.inner;

        res.on_hover_ui_at_pointer(|ui| {
            ui.label(tooltip);
            if let Some(tex) = self.ui_image_cache.get(&texture.id, ui.ctx()) {
                for t in tex {
                    ui.add(Image::from_texture(&t)
                        .max_size(Vec2::new(IMAGE_MAX_SIZE, IMAGE_MAX_SIZE))
                        .bg_fill(Color32::GRAY));
                }
            }
        });

        highlight
    }

    #[instrument(skip_all)]
    fn show_table(&mut self, ui: &mut Ui) {
        let available_width = ui.available_width();
        ui.push_id(ui.make_persistent_id("texture table"), |ui| {
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .column(Column::remainder()
                .at_least(100.0)
                .at_most(available_width - 10.0)
                .clip(true)
                .resizable(true))
            .columns(Column::initial(100.0)
                         .clip(true)
                         .resizable(true), 6)
            .column(Column::remainder().clip(true))
            .max_scroll_height(f32::MAX)
            .header(30.0, |mut row| {
                row.col(|ui| { ui.heading("Path"); });
                // row.col(|ui| { ui.heading("Type"); });
                row.col(|ui| { ui.heading("Group"); });
                row.col(|ui| { ui.heading("Instance"); });
                row.col(|ui| { ui.heading("Width"); });
                row.col(|ui| { ui.heading("Height"); });
                row.col(|ui| { ui.heading("Memory size"); });
                row.col(|ui| { ui.heading("Format"); });
                row.col(|ui| { ui.heading("Mip levels"); });
            })
            .body(|body| {
                let filtered = self.texture_list.get_filtered().clone();
                let mut highlight = None;
                body.rows(14.0, filtered.len(),
                          |mut row| {
                              let texture = &filtered[row.index()];
                              row.col(|ui| {
                                  if self.show_path_cell(
                                      texture,
                                      &texture.id.path,
                                      ui) {
                                      highlight = Some(texture.clone());
                                  }
                              });
                              // row.col(|ui| {
                              //     ui.label(format!("{:#?}", texture.id.tgi.type_id));
                              // });
                              row.col(|ui| {
                                  ui.label(format!("{:X?}", texture.id.tgi.group_id));
                              });
                              row.col(|ui| {
                                  ui.label(format!("{:X?}", texture.id.tgi.instance_id));
                              });
                              row.col(|ui| {
                                  ui.label(format!("{}", texture.width));
                              });
                              row.col(|ui| {
                                  ui.label(format!("{}", texture.height));
                              });
                              row.col(|ui| {
                                  ui.label(format!("{}", texture.memory_size));
                              });
                              row.col(|ui| {
                                  ui.label(format!("{:?}", texture.format));
                              });
                              row.col(|ui| {
                                  ui.label(format!("{}", texture.mip_levels));
                              });
                          });
                if let Some(_) = highlight {
                    self.highlighted_texture = highlight;
                }
            });
        });
    }
}

impl App for DBPFApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.update_state(ctx);

        let cur_style = (*ctx.style()).clone();
        ctx.set_style(Style {
            interaction: Interaction {
                show_tooltips_only_when_still: false,
                tooltip_delay: 0.0,
                interact_radius: 1.0,
                ..cur_style.interaction
            },
            ..cur_style
        });

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

                    self.texture_list.show_filter_menu(ui);

                    self.known_texture_menu(ctx, ui);

                    let mut show_known = self.texture_list.get_show_known();
                    ui.checkbox(&mut show_known, "Show known")
                        .changed().then(|| {
                        self.texture_list.set_show_known(show_known);
                    });

                    ui.checkbox(&mut self.show_folders, "Show paths")
                        .on_hover_text("Show what folders the packages are in?");
                });

                ui.horizontal(|ui| {
                    ui.label("Downloads: ");

                    ui.add(TextEdit::singleline(&mut self.scan_folders)
                        .id_source("scan folders")
                        .desired_width(ui.available_width() - 30.0))
                        .lost_focus().then(|| {
                        self.start_scannning(ctx);
                    });
                    if ui.button("🗁").clicked() {
                        self.open_downloads_picker();
                    }
                }).response.on_hover_text_at_pointer("The folder you want to scan (normally your downloads folder)");

                if let Some((ref path, progress, total)) = *self.find_textures_progress.lock().unwrap() {
                    ui.add(ProgressBar::new(progress as f32 / total as f32)
                        .text(Self::strip_prefix(&self.scan_ran_with_folders, path)
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
        storage.set_string("show_folders", self.show_folders.to_string());
        storage.set_string("downloads_folder", self.scan_folders.clone());

        storage.set_string("open_known_texture_gui", self.open_known_texture_gui.to_string());
        self.texture_list.save(storage);
    }
}

//noinspection ALL
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing::subscriber::set_global_default(tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
    ).expect("set up the subscriber");

    let icon = include_bytes!("../icon.png");
    let image = image::ImageReader::new(Cursor::new(icon))
        .with_guessed_format()?.decode()?;
    let buf = Vec::from(image.as_bytes());

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData {
                width: image.width(),
                height: image.height(),
                rgba: buf,
            })
            .with_drag_and_drop(true)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native("Big Awful Texture Locator",
                       native_options,
                       Box::new(|cc|
                           Ok(Box::new(DBPFApp::new(cc)))))?;
    Ok(())
}
