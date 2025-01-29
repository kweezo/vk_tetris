#version 450

layout(location = 0) in vec2 inVertex;

layout(push_constant) uniform pc{
    uint pc_texID;
    uint instance_count;
};

layout(binding = 6) uniform u_projection{
    layout(row_major) mat4 proj;
} proj; 

struct InstanceDat {
    uvec4 col;
    uvec2 pos;
};

layout(set = 0, binding = 8) readonly buffer u_instance{
    InstanceDat dat[];
} instance;


layout(location = 0) out vec2 texCoords;
layout(location = 1) out flat uint texID;
layout(location = 2) out vec4 outColor;
layout(location = 3) out uint o_instance_count;
layout(location = 4) out float o_scale_factor;

void main() {
    float scale_factor = 50;

    gl_Position = proj.proj * vec4((inVertex + instance.dat[gl_InstanceIndex].pos)*scale_factor, -0.2, 1.0); 

    outColor = instance.dat[gl_InstanceIndex].col;
    texCoords = inVertex; 
    texID = pc_texID;

    o_instance_count = instance_count;
    o_scale_factor = scale_factor;
}
