// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::async_execute;
use crate::editor::common_ui::settings::VersionInfo;
use base64::prelude::*;
use cargo_packager_updater::url::Url;
use cargo_packager_updater::{Config, Error, Update, UpdaterBuilder};
use eframe::egui::{Color32, ComboBox, ProgressBar, Ui};
use egui_inbox::{AsRequestRepaint, UiInbox, UiInboxSender};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use tracing::{error, info};

const UPDATER_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

const UPDATER_PUBLIC_KEY: &str = include_str!("../../../../data/updater/updater.pub.key");

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum ReleaseStream {
	Nightly,
	Stable,
	#[default]
	NoUpdates,
}

#[derive(Debug, Default)]
pub enum UpdaterStatus {
	#[default]
	Idle,
	Checking,
	UpdateFound(Box<Update>),
	UpdateNotFound(Option<Error>),
	Downloading {
		progress: usize,
		total_bytes: Option<u64>,
	},
	DownloadFailed(Error),
	Downloaded {
		update: Box<Update>,
		bytes: Vec<u8>,
	},
	Installing,
	InstallationFailed(Error),
	Installed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Updater {
	pub release_stream: ReleaseStream,
	pub check_update_on_startup: bool,

	#[serde(skip)]
	pub version_info: VersionInfo,

	#[serde(skip)]
	pub status: UpdaterStatus,

	#[serde(skip)]
	inbox: UiInbox<UpdaterStatus>,
}

impl Default for Updater {
	fn default() -> Self {
		Self {
			release_stream: ReleaseStream::default(),
			check_update_on_startup: true,
			version_info: VersionInfo::default(),
			status: UpdaterStatus::default(),
			inbox: UiInbox::default(),
		}
	}
}

impl Updater {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_version_info(version_info: VersionInfo) -> Self {
		Self {
			version_info,
			release_stream: version_info.release_stream(),
			..Default::default()
		}
	}

	pub fn set_version_info(&mut self, version_info: VersionInfo) {
		self.version_info = version_info;
		self.release_stream = version_info.release_stream();

		if self.check_update_on_startup {
			self.check_update();
		}
	}

	fn get_updater_config(&self) -> Config {
		let endpoints = match self.release_stream {
			ReleaseStream::Nightly => Url::parse(&format!(
				"{UPDATER_REPOSITORY}/releases/download/latest/{}-update-info.json",
				self.version_info.name,
			))
			.inspect_err(|error| error!(%error))
			.ok()
			.into_iter()
			.collect(),
			ReleaseStream::Stable => Url::parse(&format!(
				"{UPDATER_REPOSITORY}/releases/download/{}%2Flatest/update-info.json",
				self.version_info.name,
			))
			.inspect_err(|error| error!(%error))
			.ok()
			.into_iter()
			.collect(),
			ReleaseStream::NoUpdates => vec![],
		};

		Config {
			endpoints,
			pubkey: UPDATER_PUBLIC_KEY.to_string(),
			..Default::default()
		}
	}

	fn check_update(&mut self) {
		let config = self.get_updater_config();
		match UpdaterBuilder::new(self.version_info.version(), config)
			.version_comparator(|version, remote| {
				(version > remote.version) || (version.build != remote.version.build)
			})
			.build()
		{
			Err(error) => {
				error!(%error);
				self.status = UpdaterStatus::UpdateNotFound(Some(error));
			}
			Ok(updater) => {
				self.status = UpdaterStatus::Checking;

				let sender = self.inbox.sender();
				async_execute(async move {
					let update = updater.check();

					let _ = sender.send(match update {
						Ok(Some(update)) => UpdaterStatus::UpdateFound(Box::new(update)),
						Ok(None) => UpdaterStatus::UpdateNotFound(None),
						Err(error) => {
							error!(?error, msg=%error);
							UpdaterStatus::UpdateNotFound(Some(error))
						}
					});
				});
			}
		}
	}

	fn download_update(sender: UiInboxSender<UpdaterStatus>, update: Box<Update>) {
		let _ = sender.send(UpdaterStatus::Downloading {
			progress: 0,
			total_bytes: None,
		});

		async_execute(async move {
			let progress = RefCell::new(0);

			let bytes = update.download_extended(
				|bytes_read, total_bytes| {
					let mut progress = progress.borrow_mut();
					*progress += bytes_read;
					let _ = sender.send(UpdaterStatus::Downloading {
						progress: *progress,
						total_bytes,
					});
				},
				|| {},
			);

			match bytes {
				Ok(bytes) => {
					let _ = sender.send(UpdaterStatus::Downloaded { update, bytes });
				}
				Err(error) => {
					error!(?error, msg=%error);
					let _ = sender.send(UpdaterStatus::DownloadFailed(error));
				}
			}
		});
	}

	fn install_update(sender: UiInboxSender<UpdaterStatus>, update: Box<Update>, bytes: Vec<u8>) {
		let sender_clone = sender.clone();

		async_execute(async move {
			info!(?update, message = "installing");

			if let Err(error) = update.install(bytes) {
				error!(?error, msg = %error);
				let _ = sender.send(UpdaterStatus::InstallationFailed(error));
			} else {
				let _ = sender.send(UpdaterStatus::Installed);
			}
		});

		let _ = sender_clone.send(UpdaterStatus::Installing);
	}

	fn show_update_notes(ui: &mut Ui, update: &Box<Update>) {
		ui.collapsing("Changelog", |ui| {
			ui.heading(update.version.clone());
			if let Some(body) = &update.body {
				match BASE64_STANDARD.decode(body) {
					Ok(body_data) => {
						let body_string = String::from_utf8(body_data);
						match body_string {
							Ok(body) => {
								ui.label(body);
							}
							Err(error) => {
								ui.colored_label(Color32::RED, error.to_string());
							}
						}
					}
					Err(error) => {
						ui.colored_label(Color32::RED, error.to_string());
					}
				}
			}
		});
	}

	/// should always be called regardless of if the ui is actually open
	/// if this method is not called the status will not be updated
	pub fn process(&mut self, ui: &impl AsRequestRepaint) {
		self.inbox.replace(ui, &mut self.status);
	}

	pub fn show_ui(&mut self, ui: &mut Ui) -> bool {
		ui.horizontal(|ui| {
			ui.label(self.version_info.name);
			ui.label(self.version_info.full_version_string());
		});

		ui.checkbox(
			&mut self.check_update_on_startup,
			"Check for updates on startup",
		)
		.on_hover_text(
			"When checking for updates this program will download a status file from github, \
		the download statistics on this file are public information.\n\
		If you do not want to send network requests, disable this.",
		);

		let keep_open = ComboBox::new("release stream", "Release Stream")
			.selected_text(format!("{:?}", self.release_stream))
			.show_ui(ui, |ui| {
				[ReleaseStream::Stable, ReleaseStream::Nightly]
					.into_iter()
					.map(|release| {
						ui.selectable_value(
							&mut self.release_stream,
							release,
							format!("{release:?}"),
						)
					})
					.reduce(|lhs, rhs| lhs | rhs)
					.expect("reduce a fixed size list")
			})
			.inner
			.map(|res| res.clicked())
			.unwrap_or(false);

		ui.horizontal(|ui| {
			let refresh_button = ui.button("🔃");

			if refresh_button.clicked() {
				self.check_update();
			}

			match &mut self.status {
				UpdaterStatus::Idle => {
					ui.label("Check for updates");
				}
				UpdaterStatus::Checking => {
					ui.spinner();
					ui.label("Checking for updates...");
				}
				UpdaterStatus::UpdateFound(update) => {
					// TODO release notes
					ui.label("Update found!");
					let download_button = ui.button("Download");
					if download_button.clicked() {
						Self::download_update(self.inbox.sender(), update.clone());
					}
				}
				UpdaterStatus::UpdateNotFound(error) => {
					ui.label("No updates found");
					if let Some(error) = error {
						ui.colored_label(Color32::RED, error.to_string());
					}
				}
				UpdaterStatus::Downloading {
					progress,
					total_bytes,
				} => {
					ui.label("Downloading...");
					if let Some(total_bytes) = total_bytes {
						let percentage = *progress as f64 / *total_bytes as f64;
						ui.add_sized(
							[200.0, ui.style().spacing.interact_size.y],
							ProgressBar::new(percentage as f32),
						);
					} else {
						ui.label(format!("{progress} bytes"));
						ui.spinner();
					}
				}
				UpdaterStatus::DownloadFailed(error) => {
					ui.label("Download Failed: ");
					ui.colored_label(Color32::RED, error.to_string());
				}
				UpdaterStatus::Downloaded { update, bytes } => {
					ui.label("Downloaded");
					let install_button = ui.button("Install");
					if install_button.clicked() {
						Self::install_update(
							self.inbox.sender(),
							update.clone(),
							std::mem::take(bytes),
						);
					}
				}
				UpdaterStatus::Installing => {
					ui.label("Installing...");
				}
				UpdaterStatus::InstallationFailed(error) => {
					ui.label("Installation failed:");
					ui.colored_label(Color32::RED, error.to_string());
				}
				UpdaterStatus::Installed => {
					ui.label("Update installed! Restart to apply");
				}
			}
		});

		if let UpdaterStatus::UpdateFound(update) | UpdaterStatus::Downloaded { update, .. } =
			&self.status
		{
			Self::show_update_notes(ui, update);
		}

		keep_open
	}
}
