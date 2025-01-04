package main

import (
	"fmt"
	"syscall/js"
)

type Shader struct {
	context *WebGl2RenderingContext
	program js.Value
}

func NewShader(context *WebGl2RenderingContext, vertexSource, fragmentSource string) (*Shader, error) {
	vertexShader, err := createShader(context, context.VERTEX_SHADER(), vertexSource)
	if err != nil {
		return nil, fmt.Errorf("error creating vertex shader: %w", err)
	}

	fragmentShader, err := createShader(context, context.FRAGMENT_SHADER(), fragmentSource)
	if err != nil {
		context.DeleteShader(vertexShader)
		return nil, fmt.Errorf("error creating fragment shader: %w", err)
	}

	program := context.CreateProgram()
	context.AttachShader(program, vertexShader)
	context.AttachShader(program, fragmentShader)
	context.LinkProgram(program)
	context.DetachShader(program, vertexShader)
	context.DetachShader(program, fragmentShader)
	context.DeleteShader(vertexShader)
	context.DeleteShader(fragmentShader)

	status := context.GetProgramParameter(program, context.LINK_STATUS()).Bool()
	if !status {
		log := context.GetProgramInfoLog(program)
		context.DeleteProgram(program)
		return nil, fmt.Errorf("error linking shader program: %v", log)
	}

	return &Shader{
		context,
		program,
	}, nil
}

func (s *Shader) UseProgram() {
	s.context.UseProgram(s.program)
}

func (s *Shader) GetAttribLocation(name string) (int, error) {
	result := s.context.GetAttribLocation(s.program, name)
	if result < 0 {
		return 0, fmt.Errorf("no such attribute: %v", name)
	}
	return result, nil
}

func createShader(context *WebGl2RenderingContext, typ int, source string) (js.Value, error) {
	result := context.CreateShader(typ)
	context.ShaderSource(result, source)
	context.CompileShader(result)

	status := context.GetShaderParameter(result, context.COMPILE_STATUS()).Bool()
	if !status {
		log := context.GetShaderInfoLog(result)
		context.DeleteShader(result)
		return js.Null(), fmt.Errorf("error compiling shader: %v", log)
	}

	return result, nil
}
