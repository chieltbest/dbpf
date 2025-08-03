#version 330
precision mediump float;

in vec3 v_normal;
in vec2 v_texcoord;
in vec3 v_tangent;

uniform int display_mode;
uniform int dark_mode;

out vec4 out_color;

void main() {
    switch (display_mode) {
        case 0:
            out_color = vec4((v_normal + 1.0) / 2.0, 1.0);
            break;
        case 1:
            out_color = vec4((v_tangent + 1.0) / 2.0, 1.0);
            break;
        case 2:
            out_color.rb = v_texcoord;
            break;
        case 3:
            float z = pow(gl_FragCoord.z, 16) * 0.9;
            if (dark_mode != 0) {
                z = 1.0 - z;
            }
            out_color = vec4(vec3(z), 1.0);
            break;
    }
}
