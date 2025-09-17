#version 300 es
precision mediump float;

// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

in vec3 in_position;

uniform mat4 model_matrix;
uniform mat4 view_matrix;
uniform mat4 projection_matrix;

void main() {
    vec4 position = projection_matrix * view_matrix * model_matrix * vec4(in_position, 1.0);
    gl_Position = vec4(position.xy, position.z + 0.0001, position.w);
}
