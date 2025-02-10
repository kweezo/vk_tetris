#version 450

layout(push_constant) uniform pc{
    uint pc_texID;
};

layout(binding = 1) uniform sampler2D textures[3];

layout (location = 0) in vec2 texCoords;
layout (location = 1) in flat uvec3 color;
layout (location = 2) in flat uint is_pressed;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = texture(textures[pc_texID], texCoords);

    if (is_pressed == 0) {
        outColor.xyz *= 0.5;
    }
}

