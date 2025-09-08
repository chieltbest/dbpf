#version 300 es
precision mediump float;

// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

in vec3 v_normal;
in vec2 v_texcoord;
in vec3 v_tangent;

uniform int display_mode;
uniform int dark_mode;

out vec4 out_color;

void main() {
    switch (display_mode) {
        case 0: // standard
        case 1: // normals
            out_color = vec4((v_normal + 1.0) / 2.0, 1.0);
            break;
        case 2: // tangents
            out_color = vec4((v_tangent + 1.0) / 2.0, 1.0);
            break;
        case 3: // uv
            out_color = vec4(v_texcoord, 0.0, 1.0);
            break;
        case 4: // depth
            float z = sqrt((gl_FragCoord.z - 0.5) * 2.0) * 0.9 + 0.1;
            if (dark_mode == 0) {
                z = 1.0 - z;
            }
            out_color = vec4(vec3(z), 1.0);
            break;
    }
}
