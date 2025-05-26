open System
open System.Runtime.InteropServices
open System.Reflection
open Silk.NET.Maths
open Silk.NET.Windowing
open Silk.NET.OpenGL
open Silk.NET.Input
open Experiment.Graphics
open Experiment.ManifestResources

[<Struct; StructLayout(LayoutKind.Sequential)>]
type Vertex =
    val mutable Position: Vector2D<float32>
    val mutable TextureCoordinate: Vector2D<float32>
    val mutable Color: Vector4D<float32>

    new(position: Vector2D<float32>, textureCoordinate: Vector2D<float32>, color: Vector4D<float32>) =
        { Position = position
          TextureCoordinate = textureCoordinate
          Color = color }

    new(position: Vector2D<float32>, textureCoordinate: Vector2D<float32>, color: System.Drawing.Color) =
        let color =
            new Vector4D<float32>(
                float32 color.R / 255.0f,
                float32 color.G / 255.0f,
                float32 color.B / 255.0f,
                float32 color.A / 255.0f
            )

        Vertex(position, textureCoordinate, color)

let loadShaderFromManifestResources
    (gl: GL)
    (assembly: Assembly)
    (vertexShaderManifestResourceName: string)
    (fragmentShaderManifestResourceName: string)
    =
    let vertexShaderSource =
        manifestResourceString assembly vertexShaderManifestResourceName
        |> Result.mapError (fun e -> $"error loading vertex shader: {e}")

    let fragmentShaderSource =
        manifestResourceString assembly fragmentShaderManifestResourceName
        |> Result.mapError (fun e -> $"error loading fragment shader: {e}")

    match vertexShaderSource, fragmentShaderSource with
    | Ok vertexShaderSource, Ok fragmentShaderSource ->
        Shader.New gl vertexShaderSource fragmentShaderSource
        |> Result.mapError (fun e -> [ $"error creating shader: {e}" ])
    | Error vertexShaderError, Ok _ -> Error [ vertexShaderError ]
    | Ok _, Error fragmentShaderError -> Error [ fragmentShaderError ]
    | Error vertexShaderError, Error fragmentShaderError -> Error [ vertexShaderError; fragmentShaderError ]

type Shader2DTexturedColor =
    { Shader: Shader
      ProjectionMatrixUniform: int
      SamplerUniform: int }

    interface IDisposable with
        override this.Dispose() : unit = (this.Shader :> IDisposable).Dispose()


let createShader2DTexturedColor (gl: GL) =
    match
        loadShaderFromManifestResources
            gl
            (Assembly.GetExecutingAssembly())
            "Experiment.Assets.Shaders.shader.vert"
            "Experiment.Assets.Shaders.shader.frag"
    with
    | Ok shader ->
        let projectionMatrixUniform =
            shader.GetUniformLocation "projectionMatrixUniform"
            |> Result.mapError (fun e -> [ e ])

        let samplerUniform =
            shader.GetUniformLocation "samplerUniform" |> Result.mapError (fun e -> [ e ])

        match projectionMatrixUniform, samplerUniform with
        | Ok projectionMatrixUniform, Ok samplerUniform ->
            Ok
                { Shader = shader
                  ProjectionMatrixUniform = projectionMatrixUniform
                  SamplerUniform = samplerUniform }
        | Error e, Ok _ ->
            (shader :> IDisposable).Dispose()
            Error e
        | Ok _, Error e ->
            (shader :> IDisposable).Dispose()
            Error e
        | Error e1, Error e2 ->
            (shader :> IDisposable).Dispose()
            Error(List.concat [ e1; e2 ])
    | Error e -> Error e

let loadTextureFromManifestResource (gl: GL) (assembly: Assembly) (manifestResourceName: string) =
    match manifestResourceStream assembly manifestResourceName with
    | Ok stream ->
        use stream = stream
        Ok(Texture.NewFromStream gl stream)
    | Error e -> Error $"error loading texture: {e}"

let createOrthoMatrix (size: Vector2D<int>) =
    Matrix4X4.CreateOrthographicOffCenter(0.0f, float32 size.X, float32 size.Y, 0.0f, -1.0f, 1.0f)

let matrix4x4ToArray<'T
    when 'T: unmanaged
    and 'T: (new: unit -> 'T)
    and 'T: struct
    and 'T :> ValueType
    and 'T :> IFormattable
    and 'T :> IEquatable<'T>
    and 'T :> IComparable<'T>>
    (m: Matrix4X4<'T>)
    =
    [| m.M11
       m.M12
       m.M13
       m.M14
       m.M21
       m.M22
       m.M23
       m.M24
       m.M31
       m.M32
       m.M33
       m.M34
       m.M41
       m.M42
       m.M43
       m.M44 |]

type State
    private (window: IWindow, gl: GL, texture: Texture, shader: Shader2DTexturedColor, vertexArray: VertexArray<Vertex>)
    =
    let mutable orthoMatrix = Matrix4X4<float32>.Identity

    static member New (window: IWindow) (gl: GL) =
        let texture =
            loadTextureFromManifestResource gl (Assembly.GetExecutingAssembly()) "Experiment.Assets.silknet.png"

        let shader = createShader2DTexturedColor gl

        match texture, shader with
        | Ok texture, Ok shader ->
            let vertexArray =
                VertexArray.New(
                    gl,
                    { Attributes =
                        [ uint32 0,
                          VertexAttributeSpecification.FromFieldName<Vertex>
                              2
                              VertexAttribPointerType.Float
                              false
                              "Position"
                          uint32 1,
                          VertexAttributeSpecification.FromFieldName<Vertex>
                              2
                              VertexAttribPointerType.Float
                              false
                              "TextureCoordinate"
                          uint32 2,
                          VertexAttributeSpecification.FromFieldName<Vertex>
                              4
                              VertexAttribPointerType.Float
                              false
                              "Color" ]
                        |> Map.ofList },
                    ReadOnlySpan
                        [| Vertex(
                               new Vector2D<float32>(0.0f, 0.0f),
                               new Vector2D<float32>(0.0f, 0.0f),
                               System.Drawing.Color.Red
                           )
                           Vertex(
                               new Vector2D<float32>(float32 texture.Width, 0.0f),
                               new Vector2D<float32>(1.0f, 0.0f),
                               System.Drawing.Color.Green
                           )
                           Vertex(
                               new Vector2D<float32>(float32 texture.Width, float32 texture.Height),
                               new Vector2D<float32>(1.0f, 1.0f),
                               System.Drawing.Color.Blue
                           )
                           Vertex(
                               new Vector2D<float32>(0.0f, float32 texture.Height),
                               new Vector2D<float32>(0.0f, 1.0f),
                               System.Drawing.Color.Purple
                           ) |],
                    BufferUsageARB.DynamicDraw,
                    ReadOnlySpan [| uint16 0; uint16 1; uint16 2; uint16 2; uint16 3; uint16 0 |],
                    BufferUsageARB.DynamicDraw
                )

            Ok(new State(window, gl, texture, shader, vertexArray))
        | _ ->
            let mutable errors = []

            match texture with
            | Ok texture -> (texture :> IDisposable).Dispose()
            | Error e -> errors <- e :: errors

            match shader with
            | Ok shader -> (shader :> IDisposable).Dispose()
            | Error e -> errors <- List.concat [ e; errors ]

            Error errors

    interface IDisposable with
        member this.Dispose() : unit =
            (texture :> IDisposable).Dispose()
            (shader :> IDisposable).Dispose()
            (vertexArray :> IDisposable).Dispose()

    member this.Resize(size: Vector2D<int>) =
        gl.Viewport size
        orthoMatrix <- createOrthoMatrix size

    member this.Update(time: TimeSpan) = ()

    member this.Render() =
        gl.ClearColor System.Drawing.Color.CornflowerBlue
        gl.Clear ClearBufferMask.ColorBufferBit

        shader.Shader.Use()

        gl.UniformMatrix4(shader.ProjectionMatrixUniform, false, ReadOnlySpan(matrix4x4ToArray orthoMatrix))

        gl.ActiveTexture TextureUnit.Texture0
        texture.Bind()
        gl.Uniform1(shader.SamplerUniform, 0)

        vertexArray.Bind()

        gl.DrawElements(
            PrimitiveType.Triangles,
            uint32 vertexArray.IndicesLength,
            DrawElementsType.UnsignedShort,
            (nativeint 0).ToPointer()
        )

    member this.KeyDown(key: Key) = ()

    member this.KeyUp(key: Key) =
        match key with
        | Key.Escape -> window.Close()
        | _ -> ()

let mutable windowOptions = WindowOptions.Default
windowOptions.Title <- "Experiment"
windowOptions.Size <- new Vector2D<int>(1024, 768)

let window = Window.Create(windowOptions)

let mutable state = None

window.add_Load (fun () ->
    let gl = GL.GetApi window

    state <-
        Some(
            match State.New window gl with
            | Ok state ->
                state.Resize window.Size
                state
            | Error e ->
                for e in e do
                    printfn "init error: %s" e

                failwith "error initializing"
        )

    let input = window.CreateInput()

    for keyboard in input.Keyboards do
        keyboard.add_KeyDown (fun keyboard key unknown ->
            match state with
            | None -> ()
            | Some state -> state.KeyDown key)

        keyboard.add_KeyUp (fun keyboard key unknown ->
            match state with
            | None -> ()
            | Some state -> state.KeyUp key))

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
