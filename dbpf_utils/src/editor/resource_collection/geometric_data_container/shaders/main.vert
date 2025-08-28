// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

#version 300 es
precision mediump float;

in vec3 in_position;
in vec3 in_normal;
in vec2 in_texcoord;
in vec3 in_tangent;

in vec3 in_position_delta_0;
in vec3 in_position_delta_1;
in vec3 in_position_delta_2;
in vec3 in_position_delta_3;

in vec3 in_normal_delta_0;
in vec3 in_normal_delta_1;
in vec3 in_normal_delta_2;
in vec3 in_normal_delta_3;

in vec4 in_blend_keys;
in vec4 in_blend_weights;

in vec4 in_bone_keys;
in vec4 in_bone_weights;

uniform float blend_values[256];
uniform mat4 bones[256];

uniform mat4 view_matrix;
//uniform mat4 projection_matrix;

out vec3 v_normal;
out vec2 v_texcoord;
out vec3 v_tangent;

void main() {
    float morph_weights[4] = float[4](
    blend_values[int(in_blend_keys.x)] * in_blend_weights.x,
    blend_values[int(in_blend_keys.y)] * in_blend_weights.y,
    blend_values[int(in_blend_keys.z)] * in_blend_weights.z,
    blend_values[int(in_blend_keys.w)] * in_blend_weights.w);

    mat4 model_matrix = mat4(0.0);
    for (int i = 0; i < 4; i++) {
       if (int(in_bone_keys[i]) != 0xff) {
           model_matrix += in_bone_weights[i] * bones[int(in_bone_keys[i])];
       }
    }

    vec3 morph_pos_delta = in_position_delta_0 * morph_weights[0]
     + in_position_delta_1 * morph_weights[1]
     + in_position_delta_2 * morph_weights[2]
     + in_position_delta_3 * morph_weights[3];

    gl_Position = view_matrix * model_matrix * vec4(in_position + morph_pos_delta, 1.0);

    vec3 morph_norm_delta = in_normal_delta_0 * morph_weights[0]
    + in_normal_delta_1 * morph_weights[1]
    + in_normal_delta_2 * morph_weights[2]
    + in_normal_delta_3 * morph_weights[3];

    v_normal = normalize(mat3(view_matrix) * mat3(model_matrix) * (in_normal + morph_norm_delta));

    // TODO orthogonalize tangent
    v_tangent = normalize(mat3(view_matrix) * mat3(model_matrix) * in_tangent);

    v_texcoord = in_texcoord;
}
