#version 450

layout (location = 0) in vec2 texCoords;
layout (location = 1) in flat uint texID;

layout(binding = 1) uniform sampler2D textures[3];


layout(location = 0) out vec4 outColor;

void main() {
    outColor = texture(textures[texID], texCoords);
    if (outColor.xyz == vec3(0.0)) {
        discard;
    }
}


