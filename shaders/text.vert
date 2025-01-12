#version 450

layout(location = 0) in vec2 inVertex;
layout(location = 1) in uint offset;

layout(push_constant) uniform pc{
    uint pc_texID;
    uint char_count;
    float scale_factor;
    uvec2 pos;
};

layout(binding = 6) uniform proj_u{
    layout(row_major) mat4 proj;
} proj; 

layout(binding = 7) uniform offsets_u{
    vec2 offsets[93];
} offsets; 


layout(location = 0) out vec2 texCoords;
layout(location = 1) out flat uint texID;

void main() {

    float texcoord_scale_factor = 1.0f/char_count;

    texCoords = vec2(inVertex.x * texcoord_scale_factor + offset*texcoord_scale_factor, inVertex.y);

    texID = pc_texID;

    if(offset == 255) {
        gl_Position = vec4(0.0);
        return;
    }

    gl_Position = proj.proj * vec4(vec2(inVertex*scale_factor) + vec2(pos.x + gl_InstanceIndex * scale_factor, pos.y) , -0.2, 1.0); 
}
