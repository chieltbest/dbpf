// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod settings;
#[cfg(not(target_arch = "wasm32"))]
pub mod updater;
