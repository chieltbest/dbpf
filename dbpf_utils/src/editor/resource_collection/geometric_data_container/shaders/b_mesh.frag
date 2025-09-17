#version 300 es
precision mediump float;

// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

out vec4 out_color;

void main() {
    // values for premultiplied alpha
    out_color = vec4(vec3(0.25), 0.5);
}
