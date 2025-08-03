// https://stackoverflow.com/a/59739538
#version 330
precision mediump float;
uniform sampler2D t;
in vec2 uv;
out vec4 out_color;
void main() {
    out_color = texture2D(t, uv);
}