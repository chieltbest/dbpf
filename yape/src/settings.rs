// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub enum BackupOverwritePreference {
	#[default]
	Keep,
	Overwrite,
	Numbered,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize)]
pub enum DeletedRememberPreference {
	#[default]
	Forget,
	Remember,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YaPeSettings {
	pub backup_on_save: bool,

	pub backup_overwrite_preference: BackupOverwritePreference,

	pub deleted_remember_preference: DeletedRememberPreference,
}

impl Default for YaPeSettings {
	fn default() -> Self {
		Self {
			backup_on_save: true,
			backup_overwrite_preference: Default::default(),
			deleted_remember_preference: Default::default(),
		}
	}
}
