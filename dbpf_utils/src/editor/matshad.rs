// SPDX-FileCopyrightText: 2026 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::editor::Editor;
use dbpf::internal_file::material_shader::MaterialShader;
use eframe::egui::{Color32, Context, Response, ScrollArea, TextBuffer, TextEdit, TextStyle, Ui};
use egui_extras::syntax_highlighting::SyntectSettings;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};

#[derive(Default)]
pub struct MaterialShaderEditorState {
	syntect_settings: SyntectSettings,
}

impl Debug for MaterialShaderEditorState {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("MaterialShaderEditorState")
			.field("SyntaxSet", &self.syntect_settings.ps)
			.field("ThemeSet", &self.syntect_settings.ts)
			.finish()
	}
}

impl Editor for MaterialShader {
	type EditorState = MaterialShaderEditorState;

	fn new_editor(
		&self,
		_context: &Context,
		_gl_context: &Option<Arc<eframe::glow::Context>>,
	) -> Self::EditorState {
		let mut builder = SyntaxSetBuilder::new();
		builder.add(
			SyntaxDefinition::load_from_str(
				include_str!("../../../data/patterns/matShad.sublime-syntax"),
				true,
				None,
			)
			.unwrap(),
		);

		MaterialShaderEditorState {
			syntect_settings: SyntectSettings {
				ps: builder.build(),
				..Default::default()
			},
		}
	}

	fn show_editor(&mut self, state: &mut Self::EditorState, ui: &mut Ui) -> Response {
		let mut theme =
			egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
		ui.collapsing("Theme", |ui| {
			ui.group(|ui| {
				theme.ui(ui);
				theme.clone().store_in_memory(ui.ctx());
			});
		});

		let mut layouter = |ui: &Ui, buf: &dyn TextBuffer, wrap_width: f32| {
			let mut layout_job = egui_extras::syntax_highlighting::highlight_with(
				ui.ctx(),
				ui.style(),
				&theme,
				buf.as_str(),
				"matShad",
				&state.syntect_settings,
			);
			layout_job.wrap.max_width = wrap_width;
			ui.fonts_mut(|f| f.layout_job(layout_job))
		};

		// TODO proper changed response
		ScrollArea::vertical()
			.show(ui, |ui| {
				let editor = TextEdit::multiline(&mut self.text)
					.font(TextStyle::Monospace) // for cursor height
					.code_editor()
					.desired_rows(10)
					.lock_focus(true)
					.desired_width(f32::INFINITY)
					.layouter(&mut layouter);
				let background_color = if theme.is_dark() {
					Color32::BLACK
				} else {
					Color32::WHITE
				};
				let editor = editor.background_color(background_color);
				ui.add(editor)
			})
			.inner
	}
}
