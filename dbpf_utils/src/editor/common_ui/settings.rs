// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

#[cfg(not(target_arch = "wasm32"))]
use crate::editor::common_ui::updater::{ReleaseStream, Updater, UpdaterStatus};
#[cfg(not(target_arch = "wasm32"))]
use cargo_packager_updater::semver::Version;
use eframe::egui;
use eframe::egui::{Align2, DragValue, Response, RichText, Ui, Window};
#[cfg(not(target_arch = "wasm32"))]
use eframe::egui::{Color32, Separator, Vec2};
use eframe::epaint::AlphaFromCoverage;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

const STABLE_RELEASE_STRING: &str = "";

#[derive(Copy, Clone, Debug, Default)]
pub struct VersionInfo {
	pub name: &'static str,
	pub version: &'static str,
	pub release: Option<&'static str>,
}

impl VersionInfo {
	#[cfg(not(target_arch = "wasm32"))]
	pub fn release_stream(&self) -> ReleaseStream {
		match self.release {
			None => ReleaseStream::NoUpdates,
			Some(STABLE_RELEASE_STRING) => ReleaseStream::Stable,
			Some(_) => ReleaseStream::Nightly,
		}
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub fn version(&self) -> Version {
		self.full_version_string().parse().unwrap()
	}

	pub fn full_version_string(&self) -> String {
		match self.release {
			None => self.version.to_string() + "+custom",
			Some(STABLE_RELEASE_STRING) => self.version.to_string(),
			Some(release) => self.version.to_string() + "+" + release,
		}
	}
}

#[macro_export]
macro_rules! version_info {
	() => {
		VersionInfo {
			name: env!("CARGO_PKG_NAME"),
			version: env!("CARGO_PKG_VERSION"),
			release: option_env!("RELEASE"),
		}
	};
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings<T> {
	pub data: T,
	#[cfg(not(target_arch = "wasm32"))]
	updater: Updater,
	settings_open: bool,
}

impl<T> Settings<T> {
	pub fn new(user_data: T, _version_info: VersionInfo) -> Self {
		Self {
			data: user_data,
			#[cfg(not(target_arch = "wasm32"))]
			updater: Updater::with_version_info(_version_info),
			settings_open: false,
		}
	}

	pub fn init(&mut self, _version_info: VersionInfo) {
		#[cfg(not(target_arch = "wasm32"))]
		self.updater.set_version_info(_version_info);
	}

	fn menu(&mut self, ui: &mut Ui, contents: impl FnOnce(&mut Ui, &mut T) -> bool) -> bool {
		let keep_open;
		egui::widgets::global_theme_preference_buttons(ui);

		ui.horizontal(|ui| {
			ui.label("UI Scale");
			if ui.button("+").clicked() {
				egui::gui_zoom::zoom_in(ui.ctx());
			}
			if ui.button("-").clicked() {
				egui::gui_zoom::zoom_out(ui.ctx());
			}
			ui.add_enabled(
				false,
				DragValue::new(&mut ui.ctx().zoom_factor()).min_decimals(1),
			);
			ui.add_enabled(false, egui::Label::new("Ctrl +/-"));
		});

		ui.add_sized(
			ui.available_size_before_wrap().min(Vec2::new(200.0, 10.0)),
			Separator::default(),
		);

		#[cfg(not(target_arch = "wasm32"))]
		{
			keep_open = self.updater.show_ui(ui);

			ui.add_sized(
				ui.available_size_before_wrap().min(Vec2::new(200.0, 10.0)),
				Separator::default(),
			);
		}
		#[cfg(target_arch = "wasm32")]
		{
			keep_open = false;
		}

		contents(ui, &mut self.data) | keep_open
	}

	pub fn show_ui(
		&mut self,
		ui: &mut Ui,
		contents: impl FnOnce(&mut Ui, &mut T) -> bool,
	) -> Response {
		egui::global_theme_preference_switch(ui);

		#[cfg(target_arch = "wasm32")]
		let settings_text = RichText::new("⚙");
		#[cfg(not(target_arch = "wasm32"))]
		let mut settings_text = RichText::new("⚙");

		#[cfg(not(target_arch = "wasm32"))]
		{
			self.updater.process(ui);

			match self.updater.status {
				UpdaterStatus::UpdateFound(_) => {
					settings_text = settings_text.color(Color32::GREEN);
				}
				UpdaterStatus::UpdateNotFound(Some(_))
				| UpdaterStatus::DownloadFailed(_)
				| UpdaterStatus::InstallationFailed(_) => {
					settings_text = settings_text.color(Color32::RED);
				}
				_ => {}
			}
		}

		// this is a temporary workaround for the broken combobox-in-popup behaviour in egui >= 0.32
		// ref: https://github.com/emilk/egui/discussions/4463
		let settings_button = ui.button(settings_text);

		#[cfg(not(target_arch = "wasm32"))]
		if matches!(self.updater.status, UpdaterStatus::UpdateFound(_)) {
			settings_button.clone().on_hover_text("Update available!");
		}

		if settings_button.clicked() {
			self.settings_open = !self.settings_open;
		}

		let mut settings_open = self.settings_open;

		let settings_window = Window::new("Settings")
			.open(&mut settings_open)
			.collapsible(false)
			.auto_sized()
			.title_bar(false)
			.anchor(
				Align2::LEFT_TOP,
				settings_button.rect.left_bottom().to_vec2(),
			)
			.show(ui.ctx(), |ui| self.menu(ui, contents));

		if let Some(settings_window) = settings_window {
			if !settings_button.clicked()
				&& settings_window.response.clicked_elsewhere()
				&& !settings_window.inner.unwrap_or(false)
			{
				self.settings_open = false;
			}
		}

		settings_button
	}
}

impl<T> Deref for Settings<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T> DerefMut for Settings<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}
