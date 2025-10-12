#version 300 es
precision mediump float;

// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

out vec4 out_color;

void main() {
    // values for premultiplied alpha
    vec3 color = vec3(0.5);
    float alpha = 0.15;
    out_color = vec4(color * alpha, alpha);
}
