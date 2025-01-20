#version 450

layout (location = 0) in vec2 texCoords;
layout (location = 1) in flat int texID;

layout(binding = 1) uniform sampler2D textures[3];

layout(location = 0) out vec4 outColor;

in vec4 gl_FragCoord;


struct InstanceDat {
    uvec2 pos;
    uvec4 col;
};

layout(set = 0, binding = 8) readonly buffer u_instance{
    InstanceDat dat[];
} instance;

void main() {
    vec4 base_color = texture(textures[texID], texCoords) * vec4(255, 255, 255, 255);

    for (uint i = 0; i < 100; i++) {
        vec2 pos = (instance.dat[i].pos) - vec2(0.5, 0.5);
        vec4 col = instance.dat[i].col;

        vec2 dist_vec = vec2(pos.x - (gl_FragCoord.x / 50 - 1), pos.y - (gl_FragCoord.y / 50 - 1));
 
        float dist = max(0.6 - (sqrt(dist_vec.x * dist_vec.x + dist_vec.y * dist_vec.y) -0.5f), 0) * 0.05;

        base_color += dist * col[i];
    }

    outColor = base_color / vec4(255.0f, 255.0f, 255.0f, 255.0f);
}

