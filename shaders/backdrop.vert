#version 450

layout(location = 0) in vec2 inVertex;

layout(push_constant) uniform pc{
    int pc_texID;
    uint instance_count;
};

layout(binding = 6) uniform projection{
    layout(row_major) mat4 proj;
} proj; 


layout (location = 0) out vec2 texCoords;
layout (location = 1) out flat int texID;
layout (location = 2) out uint o_instance_count;

void main() {
    float scale_factor = 25;

    gl_Position = proj.proj * vec4(inVertex.x * 720, inVertex.y * 1280, -0.5, 1.0); 

    texCoords = inVertex; 
    texID = pc_texID;
    o_instance_count = instance_count;
}
