#version 330 core

in vec4 intermediateColor;

out vec4 outColor;

void main()
{
    outColor = intermediateColor;
}