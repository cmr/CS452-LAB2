#version 330
out vec4 out_color;
in vec3 Color;

void main() {
    out_color = vec4(Color, 1.0);
}
