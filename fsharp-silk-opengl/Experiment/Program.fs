open System
open System.Runtime.InteropServices
open System.Runtime.CompilerServices
open Silk.NET.Maths
open Silk.NET.Windowing
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

type VertexAttributeSpecification =
    { Size: int
      Type: VertexAttribPointerType
      Normalized: bool
      Offset: nativeint }

    static member FromFieldName<'T>
        (size: int)
        (``type``: VertexAttribPointerType)
        (normalized: bool)
        (fieldName: string)
        =
        let offset = Marshal.OffsetOf<'T> fieldName

        { Size = size
          Type = ``type``
          Normalized = normalized
          Offset = offset }

type VertexSpecification<'T> =
    { Attributes: Map<uint32, VertexAttributeSpecification> }

    member this.Stride = uint32 (Unsafe.SizeOf<'T>())

type VertexArray<'T when 'T: unmanaged and 'T: (new: unit -> 'T) and 'T: struct and 'T :> ValueType>
    private (gl: GL, vertexArray: uint32, arrayBuffer: uint32, elementArrayBuffer: uint32) =
    static member New
        (
            gl: GL,
            vertexSpecification: VertexSpecification<'T>,
            vertices: ReadOnlySpan<'T>,
            verticesUsage: BufferUsageARB,
            indices: ReadOnlySpan<uint16>,
            indicesUsage: BufferUsageARB
        ) =
        let vertexArray = gl.GenVertexArray()
        gl.BindVertexArray vertexArray
        let arrayBuffer = gl.GenBuffer()
        gl.BindBuffer(BufferTargetARB.ArrayBuffer, arrayBuffer)
        gl.BufferData<'T>(BufferTargetARB.ArrayBuffer, vertices, verticesUsage)
        let elementArrayBuffer = gl.GenBuffer()
        gl.BindBuffer(BufferTargetARB.ElementArrayBuffer, elementArrayBuffer)
        gl.BufferData<uint16>(BufferTargetARB.ElementArrayBuffer, indices, indicesUsage)

        vertexSpecification.Attributes
        |> Map.iter (fun index attribute ->
            gl.VertexAttribPointer(
                index,
                attribute.Size,
                attribute.Type,
                attribute.Normalized,
                vertexSpecification.Stride,
                attribute.Offset
            )

            gl.EnableVertexAttribArray index

            printfn
                "TODO enabled index %A, size = %A, type = %A, normalized = %A, stride = %A, offset = %A"
                index
                attribute.Size
                attribute.Type
                attribute.Normalized
                vertexSpecification.Stride
                attribute.Offset)

        new VertexArray<'T>(gl, vertexArray, arrayBuffer, elementArrayBuffer)

    interface IDisposable with
        member this.Dispose() : unit =
            gl.DeleteVertexArray vertexArray
            gl.DeleteBuffer arrayBuffer
            gl.DeleteBuffer elementArrayBuffer

    member this.Bind() = gl.BindVertexArray vertexArray

// TODO rset of VertexArray

// public uint Stride => vertexSpecification.Stride;

// public int VerticesLength => verticesLength;

// public int IndicesLength => indicesLength;

[<Struct; StructLayout(LayoutKind.Sequential)>]
type Vertex =
    val mutable Position: Vector2D<float32>

type State(gl: GL) =
    let shader =
        match
            Shader.New
                gl
                """
                #version 330 core

                layout (location = 0) in vec2 inPosition;
                // layout (location = 1) in vec2 inTextureCoordinate;
                // layout (location = 2) in vec4 inColor;

                // out vec2 intermediateTextureCoordinate;
                // out vec4 intermediateColor;

                // uniform mat4 projectionMatrixUniform;

                void main()
                {
                    gl_Position = vec4(inPosition.x, inPosition.y, 0.0, 1.0);
                    // gl_Position = projectionMatrixUniform * vec4(inPosition.x, inPosition.y, 0.0, 1.0);
                    // intermediateTextureCoordinate = inTextureCoordinate;
                    // intermediateColor = inColor;
                }
                """
                """
                #version 330 core

                // in vec2 intermediateTextureCoordinate;
                // in vec4 intermediateColor;

                out vec4 outColor;

                // uniform sampler2D samplerUniform;

                void main()
                {
                    // outColor = texture(samplerUniform, intermediateTextureCoordinate) * intermediateColor;
                    outColor = vec4(1, 1, 1, 1);
                }
                """
        with
        | Ok shader -> shader
        | Error e -> failwith e

    let vertexArray =
        VertexArray.New(
            gl,
            { Attributes =
                [ uint32 0,
                  VertexAttributeSpecification.FromFieldName<Vertex> 2 VertexAttribPointerType.Float false "Position" ]
                |> Map.ofList },
            ReadOnlySpan
                [| Vertex(Position = new Vector2D<float32>(-0.5f, -0.5f))
                   Vertex(Position = new Vector2D<float32>(0.5f, -0.5f))
                   Vertex(Position = new Vector2D<float32>(0.0f, 0.5f)) |],
            BufferUsageARB.DynamicDraw,
            (ReadOnlySpan [| uint16 0; uint16 1; uint16 2 |]),
            BufferUsageARB.DynamicDraw
        )

    interface IDisposable with
        member this.Dispose() : unit =
            (shader :> IDisposable).Dispose()
            (vertexArray :> IDisposable).Dispose()

    member this.Resize(size: Vector2D<int>) = gl.Viewport size

    member this.Update(time: TimeSpan) = ()

    member this.Render() =
        gl.ClearColor System.Drawing.Color.CornflowerBlue
        gl.Clear ClearBufferMask.ColorBufferBit

        shader.Use()
        vertexArray.Bind()
        // TODO use index array length for count
        gl.DrawElements(PrimitiveType.Triangles, uint32 3, DrawElementsType.UnsignedShort, (nativeint 0).ToPointer())

        ()

let mutable windowOptions = WindowOptions.Default
windowOptions.Title <- "Experiment"
windowOptions.Size <- new Vector2D<int>(1024, 768)

let window = Window.Create(windowOptions)

let mutable state = None

window.add_Load (fun () ->
    let gl = GL.GetApi window
    state <- Some(new State(gl)))

window.add_Closing (fun () ->
    match state with
    | None -> ()
    | Some state -> (state :> IDisposable).Dispose())

window.add_Resize (fun size ->
    match state with
    | None -> ()
    | Some state -> state.Resize size)

window.add_Update (fun time ->
    let time = TimeSpan.FromSeconds time

    match state with
    | None -> ()
    | Some state -> state.Update time)

window.add_Render (fun _ ->
    match state with
    | None -> ()
    | Some state -> state.Render())

window.Run()
