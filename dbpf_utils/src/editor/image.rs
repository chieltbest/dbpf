// SPDX-FileCopyrightText: 2026 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::Editor;
use dbpf::internal_file::image::Image;
use eframe::egui;
use eframe::egui::{
	Color32, ColorImage, Context, Frame, Rect, Response, Scene, TextureHandle, TextureOptions, Ui,
	Vec2,
};
use image::{EncodableLayout, ImageError, ImageFormat};
use std::cmp::max;
use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
enum ImageLoadError {
	#[error(transparent)]
	ImageError(#[from] ImageError),
	#[error(
		"Texture couldn't be loaded because it is too big ({0} px), try reloading the editor?"
	)]
	TooBig(u32),
}

pub struct ImageEditorState {
	texture: Option<Result<TextureHandle, ImageLoadError>>,
	format: Option<ImageFormat>,
	zoom_state: Rect,
}

impl Default for ImageEditorState {
	fn default() -> Self {
		Self {
			texture: None,
			format: None,
			zoom_state: Rect::ZERO,
		}
	}
}

impl Debug for ImageEditorState {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ImageEditorState")
			.field(
				"texture",
				&self.texture.as_ref().map(|t| {
					t.as_ref()
						.map(|th| format!("{} {:?}", th.name(), th.size()))
				}),
			)
			.field("zoom_state", &self.zoom_state)
			.finish()
	}
}

impl Editor for Image {
	type EditorState = ImageEditorState;

	fn new_editor(
		&self,
		context: &Context,
		_gl_context: &Option<Arc<eframe::glow::Context>>,
	) -> Self::EditorState {
		let cursor = Cursor::new(&self.data);
		let mut image_reader = image::ImageReader::new(cursor);
		image_reader.set_format(ImageFormat::Tga);
		let image_reader = image_reader
			.with_guessed_format()
			.expect("cursor io never fails");
		let format = image_reader.format();
		let decoded = image_reader.decode();

		let texture = decoded.map_err(|err| err.into()).and_then(|img| {
			let max_texture_side = context.input(|input| input.max_texture_side) as u32;
			if img.width() > max_texture_side || img.height() > max_texture_side {
				return Err(ImageLoadError::TooBig(max(img.width(), img.height())));
			}

			Ok(context.load_texture(
				"image",
				ColorImage::from_rgba_unmultiplied(
					[img.width() as usize, img.height() as usize],
					img.to_rgba8().as_bytes(),
				),
				TextureOptions::NEAREST,
			))
		});

		ImageEditorState {
			texture: Some(texture),
			format,
			zoom_state: Rect::ZERO,
		}
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		ui.label(if let Some(f) = state.format {
			format!("{:?}", f)
		} else {
			"Unknown format".to_string()
		});

		Frame::group(ui.style())
			.show(ui, |ui| {
				let scene =
					Scene::new()
						.zoom_range(0.1..=16.0)
						.show(ui, &mut state.zoom_state, |ui| match &state.texture {
							Some(Ok(texture)) => {
								let size = texture.size();
								ui.add(
									egui::Image::new(texture).fit_to_exact_size(Vec2::new(
										size[0] as f32,
										size[1] as f32,
									)),
								);
							}
							Some(Err(err)) => {
								ui.colored_label(Color32::RED, format!("{err}"));
							}
							_ => {}
						});
				if scene.response.double_clicked() {
					state.zoom_state = Rect::ZERO;
				}
			})
			.response
	}
}
