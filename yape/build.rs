// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
	embed_resource::compile_for("res/yape.rc", ["yape"], embed_resource::NONE)
		.manifest_required()
		.unwrap();
}
