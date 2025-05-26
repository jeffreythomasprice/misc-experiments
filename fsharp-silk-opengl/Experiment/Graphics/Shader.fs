namespace Experiment.Graphics

open System
open Silk.NET.OpenGL

type Shader private (gl: GL, program: uint32, vertexShader: uint32, fragmentShader: uint32) =
    interface IDisposable with
        member this.Dispose() : unit =
            gl.DeleteShader vertexShader
            gl.DeleteShader fragmentShader
            gl.DeleteProgram program

    static member New (gl: GL) (vertexSource: string) (fragmentSource: string) =
        match Shader.CreateShader gl ShaderType.VertexShader vertexSource with
        | Ok vertexShader ->
            match Shader.CreateShader gl ShaderType.FragmentShader fragmentSource with
            | Ok fragmentShader ->
                let program = gl.CreateProgram()
                gl.AttachShader(program, vertexShader)
                gl.AttachShader(program, fragmentShader)
                gl.LinkProgram program

                if gl.GetProgram(program, ProgramPropertyARB.LinkStatus) = 0 then
                    let log = gl.GetProgramInfoLog program
                    gl.DeleteShader vertexShader
                    gl.DeleteShader fragmentShader
                    gl.DeleteProgram program
                    Error log
                else
                    Ok(new Shader(gl, program, vertexShader, fragmentShader))
            | Error e ->
                gl.DeleteShader vertexShader
                Error e
        | Error e -> Error e

    static member private CreateShader (gl: GL) (shaderType: ShaderType) (source: string) =
        let result = gl.CreateShader shaderType
        gl.ShaderSource(result, source)
        gl.CompileShader result

        if gl.GetShader(result, ShaderParameterName.CompileStatus) = 0 then
            let log = gl.GetShaderInfoLog result
            gl.DeleteShader result
            Error log
        else
            Ok result

    member this.Use() = gl.UseProgram program
