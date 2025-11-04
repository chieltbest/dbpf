// SPDX-FileCopyrightText: 2023-2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
	cmp::min,
	fmt::{Debug, Formatter, Write},
	io::Cursor,
	sync::Arc,
};

use binrw::BinRead;
use dbpf::internal_file::resource_collection::texture_resource::{
	decoded_texture::{DecodedTexture, ShrinkDirection},
	KnownPurpose, Purpose, TextureFormat, TextureResource, TextureResourceData,
};
use eframe::egui::Color32;
use eframe::{
	egui,
	egui::{
		Button, ColorImage, ComboBox, DragValue, Pos2, Rect, Response, Slider, TextureOptions, Ui,
	},
	glow,
};
use enum_iterator::all;
use futures::channel::oneshot;
use image::ImageReader;
use rfd::FileHandle;
use tracing::error;

use crate::{
	async_execute,
	editor::{
		r#enum::{EnumEditor, EnumEditorState},
		Editor,
	},
};

impl EnumEditor for Purpose {
	type KnownEnum = KnownPurpose;

	fn from_known(known_enum: &Self::KnownEnum) -> Self {
		Self::Known(*known_enum)
	}

	fn from_string(string: &String) -> Option<Self>
	where
		Self: Sized,
	{
		let f: f32 = string.parse().ok()?;
		let bytes = f.to_le_bytes();
		Purpose::read_le(&mut Cursor::new(&bytes)).ok()
	}

	fn known_name(known_enum: &Self::KnownEnum) -> String {
		format!("{known_enum:?}")
	}

	fn full_name(&self) -> String {
		match self {
			Self::Known(known) => Self::known_name(known),
			Self::Unknown(i) => format!("{i:?}"),
		}
	}

	fn known_hover_string(known_enum: &Self::KnownEnum) -> String {
		let mut str = String::new();
		writeln!(str, "{:?}", known_enum).unwrap();
		write!(str, "Id: {:?}", f32::from(*known_enum)).unwrap();
		str
	}

	fn hover_string(&self) -> Option<String> {
		match self {
			Self::Known(known) => Some(Self::known_hover_string(known)),
			Self::Unknown(_) => None,
		}
	}

	fn search_strings(known_enum: &Self::KnownEnum) -> Vec<String> {
		vec![
			format!("{known_enum:?}"),
			format!("{}", f32::from(*known_enum)),
		]
	}

	fn all_known() -> impl Iterator<Item = Self::KnownEnum> {
		all::<KnownPurpose>()
	}
}

#[derive(Default)]
pub struct TextureResourceEditorState {
	textures: Vec<Vec<Option<egui::TextureHandle>>>,
	zoom_state: Vec<(Rect, usize)>,
	original_texture_bgra: TextureResource,
	preserve_transparency: u8,
	save_file_picker: Option<oneshot::Receiver<Option<FileHandle>>>,
	enum_editor_state: EnumEditorState,
	alpha_texture_color: Color32,
}

impl Debug for TextureResourceEditorState {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_list()
			.entries(self.textures.iter().map(|texture| {
				texture
					.iter()
					.map(|mip| {
						mip.as_ref()
							.map(|img| format!("{}: {:?}", img.name(), img.size_vec2()))
					})
					.collect::<Vec<_>>()
			}))
			.finish()
	}
}

impl TextureResourceEditorState {
	fn load_textures(
		res: &TextureResource,
		context: &egui::Context,
	) -> Vec<Vec<Option<egui::TextureHandle>>> {
		let source_format = res.get_format();

		res.decompress_all()
			.into_iter()
			.enumerate()
			.map(|(tex_num, texture)| {
				texture
					.into_iter()
					.enumerate()
					.rev()
					.map(|(mip_num, mip)| {
						mip.inspect_err(|err| error!(?err)).ok().map(|mut decoded| {
							if source_format == TextureFormat::Alpha {
								// display alpha as tinted white
								decoded.data.chunks_exact_mut(4).for_each(|c| {
									c[0] = 0xff;
									c[1] = 0xff;
									c[2] = 0xff;
								});
							}

							context.load_texture(
								format!("texture{tex_num}_mip{mip_num}"),
								ColorImage::from_rgba_unmultiplied(
									[decoded.width, decoded.height],
									&decoded.data,
								),
								TextureOptions::NEAREST,
							)
						})
					})
					.collect()
			})
			.collect()
	}

	fn refresh_textures_from(&mut self, res: &TextureResource, context: &egui::Context) {
		self.textures = Self::load_textures(res, context);
		self.zoom_state.resize(self.textures.len(), (Rect::ZERO, 0));
		let mip_levels = res.mip_levels() - 1;
		self.zoom_state
			.iter_mut()
			.for_each(|(_r, mip)| *mip = min(*mip, mip_levels))
	}
}

impl Editor for TextureResource {
	type EditorState = TextureResourceEditorState;

	fn new_editor(
		&self,
		context: &egui::Context,
		_gl: &Option<Arc<glow::Context>>,
	) -> Self::EditorState {
		let mut new = Self::EditorState {
			original_texture_bgra: self
				.recompress_with_format(TextureFormat::RawARGB32)
				.unwrap(),
			alpha_texture_color: Color32::from_rgb(127, 125, 120),
			..Default::default()
		};
		new.refresh_textures_from(self, context);
		new
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		if let Some(picker) = &mut state.save_file_picker {
			if let Ok(Some(handle)) = picker.try_recv() {
				state.save_file_picker = None;
				if let Some(handle) = handle {
					let mut cur = Cursor::new(vec![]);
					if let Ok(()) = self.export_dds(&mut cur) {
						let res = futures::executor::block_on(handle.write(&cur.into_inner()));
						if let Err(e) = res {
							error!(?e);
						}
					}
				}
			}
		}

		let mut update_images = false;

		let mut res = self.file_name.name.show_editor(&mut 500.0, ui);
		ui.horizontal_wrapped(|ui| {
			res |= ui.add_enabled(false, DragValue::new(&mut self.width));
			ui.label("width");
			res |= ui.add_enabled(false, DragValue::new(&mut self.height));
			ui.label("height");
		});
		ui.horizontal_wrapped(|ui| {
            res |= ui.add_enabled(false, DragValue::new(
                &mut self.textures.first()
                    .map(|t| t.entries.len())
                    .unwrap_or(0)));
            ui.label("Mip levels");

            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut state.preserve_transparency, 0..=255)) |
                    ui.label("Preserve transparency")
            }).inner.on_hover_text("makes sure that thin objects with transparency also show up correctly in mipmaps\n\
            This option is intended for textures that use alpha testing, NOT for textures that have AlphaBlendMode set to \"blend\".\n\n\
            To use this option, set the texture format to DXT1 and have a look at the mipmaps; does the texture show up correctly in all mipmaps? \
            If not try adjusting this value, click \"Recalculate all mipmaps\" and look again.\n\
            Don't forget to set the format back to DXT5 after you're done!");
        });
		let preserve_transparency = if state.preserve_transparency > 0 {
			Some(state.preserve_transparency)
		} else {
			None
		};

		ui.horizontal_wrapped(|ui| {
            let top_is_lifo = self
                .textures
                .iter()
                .any(|tex| matches!(tex.entries.last(), Some(TextureResourceData::LIFOFile { .. })));
            let bottom_is_lifo = self
                .textures
                .iter()
                .any(|tex| matches!(tex.entries.first(), Some(TextureResourceData::LIFOFile { .. })));

            if ui
                .add_enabled(!top_is_lifo, Button::new("Recalculate all mipmaps"))
                .clicked()
            {
                state.original_texture_bgra.remove_smaller_mip_levels();
                state.original_texture_bgra.add_max_mip_levels(preserve_transparency);
                self.remove_smaller_mip_levels();
                self.add_max_mip_levels(preserve_transparency);
                res.mark_changed();
                update_images = true;
            }
            if ui
                .add_enabled(
                    self.mip_levels() < self.max_mip_levels() && !bottom_is_lifo,
                    Button::new("Add missing mipmaps"),
                )
                .clicked()
            {
                state.original_texture_bgra.add_max_mip_levels(preserve_transparency);
                self.add_max_mip_levels(preserve_transparency);
                res.mark_changed();
                update_images = true;
            }
            if ui
                .add_enabled(self.mip_levels() > 1, Button::new("Remove all mipmaps"))
                .clicked()
            {
                state.original_texture_bgra.remove_smaller_mip_levels();
                self.remove_smaller_mip_levels();
                res.mark_changed();
                update_images = true;
            }
            if ui
                .add_enabled(self.mip_levels() > 1, Button::new("Remove largest texture"))
                .on_hover_text("Deletes the biggest image from the list of mipmaps, effectively halving the image size")
                .clicked()
            {
                state.original_texture_bgra.remove_largest_mip_levels(1);
                self.remove_largest_mip_levels(1);
                for (zoom, mip_i) in &mut state.zoom_state {
                    *zoom = *zoom / 2.0;
                    *mip_i = mip_i.saturating_sub(1);
                }
                res.mark_changed();
                update_images = true;
            }
        });

		ui.horizontal_wrapped(|ui| {
			for (shrink_direction, text, tooltip) in [
				(
					ShrinkDirection::Both,
					"Shrink",
					"Shrink the texture in both dimensions by 2x",
				),
				(
					ShrinkDirection::Horizontal,
					"Shrink horizontally",
					"Shrink the texture horizontally by 2x",
				),
				(
					ShrinkDirection::Vertical,
					"Shrink vertically",
					"Shrink the texture vertically by 2x",
				),
			] {
				if ui
					.add_enabled(self.can_shrink(shrink_direction), Button::new(text))
					.on_hover_text(tooltip)
					.clicked()
				{
					let _ = state
						.original_texture_bgra
						.shrink(preserve_transparency, shrink_direction);
					let _ = self.shrink(preserve_transparency, shrink_direction);
					for (zoom, mip_i) in &mut state.zoom_state {
						*zoom = match shrink_direction {
							ShrinkDirection::Both => *zoom / 2.0,
							ShrinkDirection::Horizontal => Rect::from_min_max(
								Pos2::new(zoom.min.x / 2.0, zoom.min.y),
								Pos2::new(zoom.max.x / 2.0, zoom.max.y),
							),
							ShrinkDirection::Vertical => Rect::from_min_max(
								Pos2::new(zoom.min.x, zoom.min.y / 2.0),
								Pos2::new(zoom.max.x, zoom.max.y / 2.0),
							),
						};
						*mip_i = min(*mip_i, state.original_texture_bgra.mip_levels());
					}
					res.mark_changed();
					update_images = true;
				}
			}
		});

		let formats = [
			TextureFormat::DXT5,
			TextureFormat::DXT3,
			TextureFormat::DXT1,
			TextureFormat::Alpha,
			TextureFormat::Grayscale,
			TextureFormat::RawARGB32,
			TextureFormat::RawRGB24,
			TextureFormat::AltARGB32,
			TextureFormat::AltRGB24,
		];
		let mut current_format = self.get_format();
		ui.horizontal_wrapped(|ui| {
            let cbres = ComboBox::new("format", "Texture Format")
                .selected_text(format!("{:?}", current_format))
                .show_ui(ui, |ui| {
                    formats
                        .map(|format| ui.selectable_value(&mut current_format, format, format!("{:?}", format)))
                        .into_iter()
                        .reduce(|r1, r2| r1 | r2)
                        .unwrap()
                });
            if let Some(inner) = cbres.inner {
                if inner.changed() {
                    if let Ok(mut new) = state.original_texture_bgra.recompress_with_format(current_format) {
                        new.file_name = std::mem::take(&mut self.file_name);
                        new.file_name_repeat = std::mem::take(&mut self.file_name_repeat);
                        new.purpose = self.purpose;
                        new.unknown = self.unknown;
                        *self = new;
                    }
                    update_images = true;

	                res.mark_changed();
                }
            }

			if matches!(current_format, TextureFormat::Alpha) {
				egui::widgets::color_picker::color_edit_button_srgba(
					ui,
					&mut state.alpha_texture_color,
					egui::widgets::color_picker::Alpha::Opaque);
			}

            if ui
                .button("Replace original")
                .on_hover_text(
                    "apply the change in texture format to the original texture\n\n\
                YaPe will always remember the original texture so you can easily switch back to it. \
                If you want to replace this texture with the currently visible one use this button.",
                )
                .clicked()
            {
                if let Ok(argb) = self.recompress_with_format(TextureFormat::RawARGB32) {
                    state.original_texture_bgra = argb;
                }
            }
        });

		if update_images {
			state.refresh_textures_from(self, ui.ctx());
			update_images = false;
		}

		ui.horizontal(|ui| {
			self.purpose
				.show_enum_editor(&mut state.enum_editor_state, ui);
			ui.label("Purpose");
		});

		if ui
			.button("Export DDS")
			.on_hover_text("export the currently visible texture and all mipmaps to a .dds file")
			.clicked()
			&& state.save_file_picker.is_none()
		{
			let (tx, rx) = oneshot::channel();
			let dialog = rfd::AsyncFileDialog::new()
				.set_file_name(format!(
					"{}.dds",
					String::from_utf8_lossy(&self.file_name.name.0.data)
				))
				.add_filter("DirectDraw Surface", &["dds"]);
			// TODO global options open file path set directory
			let dialog = dialog.save_file();
			async_execute(async move {
				let file = dialog.await;
				let _ = if let Some(handle) = file {
					tx.send(Some(handle))
				} else {
					tx.send(None)
				};
			});
			state.save_file_picker = Some(rx);
		}

		let mip_level_names = (0..self.mip_levels())
			.map(|mip| {
				let (w, h) = self.mip_size(mip);
				format!("{w}x{h}")
			})
			.enumerate()
			.collect::<Vec<_>>();
		let original_size = self.mip_size(0);

		for (texture_num, (texture, (zoom, cur_selected_mip_level))) in self
			.textures
			.iter_mut()
			.zip(&mut state.zoom_state)
			.enumerate()
		{
			ui.separator();

			ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut texture.creator_id).hexadecimal(8, false, true)) | ui.label("Creator ID")
            })
            .inner
            .on_hover_text(
                "Creator ID of the creator of this texture\n\
                    If the texture has not been uploaded to online services the creator ID will be either \
                    FF000000 or FFFFFFFF",
            );

			ui.horizontal_wrapped(|ui| {
				if ui.button("Reset zoom").clicked() {
					*zoom = Rect::ZERO;
				}
				for (mip_i, name) in mip_level_names.clone() {
					ui.radio_value(cur_selected_mip_level, mip_i, name);
				}
			});

			egui::Frame::group(ui.style()).show(ui, |ui| {
				egui::Scene::new()
					.zoom_range(0.1..=16.0)
					.show(ui, zoom, |ui| {
						if let Some(mip_level) = texture.entries.get(*cur_selected_mip_level) {
							match mip_level {
								TextureResourceData::Embedded(_) => {
									if let Some(image) = state.textures[texture_num]
										[*cur_selected_mip_level]
										.as_ref()
									{
										ui.add(
											egui::Image::new(image)
												.tint(match current_format {
													TextureFormat::Alpha => {
														state.alpha_texture_color
													}
													_ => Color32::WHITE,
												})
												.fit_to_exact_size(egui::Vec2::new(
													original_size.0 as f32,
													original_size.1 as f32,
												)),
										);
									}
								}
								TextureResourceData::LIFOFile { file_name } => {
									ui.end_row();
									ui.label(format!(
										"file: {}",
										String::from_utf8_lossy(&file_name.0.data)
									));
								}
							}
						}
					});
			});
		}

		ui.input_mut(|i| {
			if let Some(file) = i.raw.dropped_files.first().cloned() {
				if file.name.ends_with(".dds")
					|| file
						.path
						.as_ref()
						.is_some_and(|p| p.extension() == Some("dds".as_ref()))
				{
					match if let Some(path) = file.path {
						match std::fs::read(path) {
							Err(e) => {
								error!(?e);
								None
							}
							Ok(bytes) => {
								let mut cur = Cursor::new(bytes);
								Some(TextureResource::import_dds(&mut cur))
							}
						}
					} else if let Some(bytes) = file.bytes {
						let mut cur = Cursor::new(&bytes);
						Some(TextureResource::import_dds(&mut cur))
					} else {
						None
					} {
						Some(Err(e)) => error!(?e),
						Some(Ok(mut texture)) => {
							texture.file_name = self.file_name.clone();
							texture.unknown = self.unknown;
							texture.purpose = self.purpose;
							texture.file_name_repeat = self.file_name_repeat.clone();

							if let Ok(bgra) =
								texture.recompress_with_format(TextureFormat::RawARGB32)
							{
								state.original_texture_bgra = bgra;
							}
							*self = texture;
							update_images = true;
						}
						_ => {}
					}
					i.raw.dropped_files.clear();
					return;
				}

				let image = if let Some(path) = file.path {
					ImageReader::open(path).ok().map(|i| i.decode())
				} else {
					file.bytes
						.and_then(|bytes| {
							ImageReader::new(Cursor::new(bytes))
								.with_guessed_format()
								.ok()
						})
						.map(|i| i.decode())
				};

				if let Some(Ok(image)) = image {
					let orig_format = self.get_format();

					let has_mip = self.mip_levels() > 1;

					self.compress_replace(
						DecodedTexture {
							width: image.width() as usize,
							height: image.height() as usize,
							data: image.into_rgba8().to_vec(),
						},
						Some(TextureFormat::RawARGB32),
					);

					if has_mip {
						self.add_max_mip_levels(preserve_transparency);
					}

					state.original_texture_bgra = self.clone();

					if let Ok(new) = self.recompress_with_format(orig_format) {
						*self = new;
					}

					update_images = true;

					i.raw.dropped_files.clear();
				}
			}
		});

		if update_images {
			state.refresh_textures_from(self, ui.ctx());
		}

		res
	}
}
