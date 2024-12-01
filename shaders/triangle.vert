#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;

layout(binding = 8) uniform TransformStruct{
    mat2 model[1];
};

layout(push_constant) uniform _PushConstant{
    int modelID;
};

void main() {
    gl_Position =vec4(model[modelID] * inPosition, 0.0, 1.0);
    fragColor = fragColor;
}
