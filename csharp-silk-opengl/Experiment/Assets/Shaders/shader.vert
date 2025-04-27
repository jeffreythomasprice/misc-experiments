#version 330 core

layout (location = 0) in vec2 inPosition;
layout (location = 1) in vec2 inTextureCoordinate;
layout (location = 2) in vec4 inColor;

out vec2 intermediateTextureCoordinate;
out vec4 intermediateColor;

uniform mat4 projectionMatrixUniform;

void main()
{
	gl_Position = projectionMatrixUniform * vec4(inPosition.x, inPosition.y, 0.0, 1.0);
	intermediateTextureCoordinate = inTextureCoordinate;
	intermediateColor = inColor;
}
