#version 450

layout(location = 0) in vec2 inVertex;

layout(location = 1) in uvec2 inPosition;
layout(location = 2) in uvec3 inColor;

layout(push_constant) uniform pc{
    int pc_texID;
};

layout(binding = 6) uniform projection{
    layout(row_major) mat4 proj;
} proj; 


layout(location = 0) out vec2 texCoords;
layout(location = 1) out flat int texID;
layout(location = 2) out vec3 outColor;

void main() {
    float scale_factor = 25;

    gl_Position = proj.proj * vec4((inVertex + inPosition)*scale_factor, 0.0, 1.0); 

    outColor = inColor;
    texCoords = inVertex; 
    texID = pc_texID;
}
