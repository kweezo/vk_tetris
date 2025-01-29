#version 450

//layout (location = 0) in vec2 texCoords;
layout (location = 2) in flat uvec3 color;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(color, 1.0);
}

