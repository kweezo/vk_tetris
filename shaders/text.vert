#version 450

layout(location = 0) in vec2 inVertex;
layout(location = 1) in uint offset;
layout(location = 2) in float padding;

layout(push_constant) uniform pc{
    uint pc_texID;
    uint char_count;
    uvec2 pos;
    uvec2 p_size;
    uint instance_count;
    uint chars_per_row;
    uint row_count;
};

layout(binding = 6) uniform proj_u{
    layout(row_major) mat4 proj;
} proj; 


layout(location = 0) out vec2 texCoords;
layout(location = 1) out flat uint texID;

void main() {

    float texcoord_scale_factor = 1.0f/chars_per_row;

    texCoords = vec2(
        inVertex.x * texcoord_scale_factor + (offset % chars_per_row) * texcoord_scale_factor,
        floor(offset / chars_per_row) / row_count + inVertex.y/row_count); 

    texID = pc_texID;

    if(offset == 255) {
        gl_Position = vec4(0.0);
        return;
    }

    vec2 size = vec2(p_size.x / float(instance_count), p_size.y);

    vec2 spaced_coords = vec2(inVertex * size) + vec2(pos.x + gl_InstanceIndex * size.x - padding * size.x, pos.y); 


    gl_Position = proj.proj * vec4(spaced_coords,
     -0.3, 1.0); 
}
