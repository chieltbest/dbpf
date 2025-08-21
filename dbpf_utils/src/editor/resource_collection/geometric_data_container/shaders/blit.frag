#version 300 es
precision mediump float;

// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

uniform sampler2D t;
in vec2 uv;
out vec4 out_color;
void main() {
    out_color = texture(t, uv);
}
