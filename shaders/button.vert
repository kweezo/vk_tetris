#version 450

layout(location = 0) in vec2 inVertex;

layout (location = 1) in uvec2 pos;
layout (location = 2) in uvec2 size;
layout (location = 3) in uvec3 color;

layout (location = 4) in uint is_pressed;

layout(binding = 6) uniform projection{
    layout(row_major) mat4 proj;
} proj; 


layout (location = 0) out vec2 texCoords;
layout (location = 1) out flat uvec3 o_color;
layout (location = 2) out flat uint o_is_pressed;

void main() {
    gl_Position = proj.proj * vec4((inVertex * size) + pos, -0.2, 1.0); 

    texCoords = inVertex; 
    o_color = color;
    o_is_pressed= is_pressed;
}
