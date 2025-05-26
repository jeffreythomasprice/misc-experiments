#version 330 core

in vec2 intermediateTextureCoordinate;
in vec4 intermediateColor;

out vec4 outColor;

uniform sampler2D samplerUniform;

void main()
{
    outColor = texture(samplerUniform, intermediateTextureCoordinate) * intermediateColor;
}