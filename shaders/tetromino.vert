#version 450

layout(location = 0) in vec2 inVertex;

layout(location = 1) in uvec2 inPosition;
layout(location = 2) in uvec3 inColor;

layout(push_constant) uniform pc{
    int pc_texID;
};


layout(location = 0) out vec2 texCoords;
layout(location = 1) out flat int texID;
layout(location = 2) out vec3 outColor;

void main() {
    gl_Position =vec4(inVertex + inPosition, 0.0, 1.0);

    outColor = inColor;
    texCoords = inVertex + vec2(0.5, 0.5);
    texID = pc_texID;
}
