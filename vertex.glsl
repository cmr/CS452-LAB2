#version 330

in vec2 position;
in vec3 color;
uniform vec3 const_color;
out vec3 Color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    if (all(equal(const_color, vec3(0.0)))) {
        Color = color;
    } else {
        Color = const_color;
    }
}
