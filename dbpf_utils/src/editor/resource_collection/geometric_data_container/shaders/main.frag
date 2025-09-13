#version 300 es
precision mediump float;

// SPDX-FileCopyrightText: 2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

in vec3 v_position;
in vec3 v_normal;
in vec2 v_texcoord;
in vec3 v_tangent;

uniform int display_mode;
uniform int dark_mode;

out vec4 out_color;

void main() {
    switch (display_mode) {
        case 0: // standard
            // blinn-phong with the light coming from behind the camera
            vec3 object_color = vec3(0.5);

            float ambient = 0.3;
            float diffuse_strength = 1.0;
            float specular_strength = 0.1;
            float specular_exponent = 32.0;
            if (dark_mode == 0) {
                ambient = 0.8;
                diffuse_strength = 0.8;
                specular_strength = 0.05;
            }

            vec3 light_pos = vec3(0.0, 1.0, -1.0);
            vec3 light_dir = normalize(light_pos - v_position);
            vec3 camera_pos = vec3(0.0);
            vec3 view_dir = normalize(camera_pos - v_position);

            float diffuse = max(dot(v_normal, light_dir), 0.0) * diffuse_strength;

            vec3 halfwaydir = normalize(view_dir + light_dir);
            float specular = pow(max(dot(v_normal, halfwaydir), 0.0), specular_exponent) * specular_strength;

            out_color = vec4(object_color * ambient + object_color * diffuse + vec3(specular), 1.0);
            break;
        case 1: // normals
            out_color = vec4((v_normal * vec3(1.0, 1.0, -1.0) + 1.0) / 2.0, 1.0);
            break;
        case 2: // tangents
            out_color = vec4((v_tangent * vec3(1.0, 1.0, -1.0) + 1.0) / 2.0, 1.0);
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
