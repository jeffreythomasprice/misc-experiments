#version 450

layout(binding = 0) uniform UniformMatrices {
    mat4 model;
    mat4 view;
    mat4 projection;
} uniformMatrices;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTextureCoordinate;
layout(location = 2) in vec4 inColor;

layout(location = 0) out vec2 fragTextureCoordinate;
layout(location = 1) out vec4 fragColor;

void main() {
    gl_Position = uniformMatrices.projection * uniformMatrices.view * uniformMatrices.model * vec4(inPosition, 0.0, 1.0);
    fragTextureCoordinate = inTextureCoordinate;
    fragColor = inColor;
}