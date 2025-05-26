open System
open System.Reflection
open System.Runtime.InteropServices
open Silk.NET.Maths
open Silk.NET.Windowing
open Silk.NET.OpenGL
open Silk.NET.Input
open Experiment.Graphics
open System.IO

let sanitizeName (name: string) =
    name.Replace('/', '*').Replace('-', '*').Replace('_', '*').Replace('.', '*').ToLower()

let sanitizeManifestResourceName (assembly: Assembly) (name: string) =
    let sanitizedName = sanitizeName name

    match
        assembly.GetManifestResourceNames()
        |> Array.filter (fun resourceName -> sanitizedName = sanitizeName resourceName)
    with
    | [||] -> Error $"no results for {name}"
    | [| result |] -> Ok result
    | _ -> Error $"multiple results for {name}"

let manifestResourceStream (assembly: Assembly) (name: string) =
    sanitizeManifestResourceName assembly name
    |> Result.bind (fun name ->
        match assembly.GetManifestResourceStream name with
        | null -> Error $"failed to find embedded file: {name}"
        | result -> Ok result)

let manifestResourceString (assembly: Assembly) (name: string) =
    manifestResourceStream assembly name
    |> Result.bind (fun stream ->
        use stream = stream
        use reader = new StreamReader(stream)
        Ok(reader.ReadToEnd()))

[<Struct; StructLayout(LayoutKind.Sequential)>]
type Vertex =
    val mutable Position: Vector2D<float32>
    val mutable Color: Vector4D<float32>

    new(position: Vector2D<float32>, color: Vector4D<float32>) = { Position = position; Color = color }

    new(position: Vector2D<float32>, color: System.Drawing.Color) =
        let color =
            new Vector4D<float32>(
                float32 color.R / 255.0f,
                float32 color.G / 255.0f,
                float32 color.B / 255.0f,
                float32 color.A / 255.0f
            )

        Vertex(position, color)

let loadShaderFromManifestResources
    (gl: GL)
    (vertexShaderManifestResourceName: string)
    (fragmentShaderManifestResourceName: string)
    =
    let vertexShaderSource =
        manifestResourceString (Assembly.GetExecutingAssembly()) vertexShaderManifestResourceName
        |> Result.mapError (fun e -> $"error loading vertex shader: {e}")

    let fragmentShaderSource =
        manifestResourceString (Assembly.GetExecutingAssembly()) fragmentShaderManifestResourceName
        |> Result.mapError (fun e -> $"error loading fragment shader: {e}")

    match vertexShaderSource, fragmentShaderSource with
    | Ok vertexShaderSource, Ok fragmentShaderSource ->
        Shader.New gl vertexShaderSource fragmentShaderSource
        |> Result.mapError (fun e -> [ $"error creating shader: {e}" ])
    | Error vertexShaderError, Ok _ -> Error [ vertexShaderError ]
    | Ok _, Error fragmentShaderError -> Error [ fragmentShaderError ]
    | Error vertexShaderError, Error fragmentShaderError -> Error [ vertexShaderError; fragmentShaderError ]

type State private (window: IWindow, gl: GL, shader: Shader, vertexArray: VertexArray<Vertex>) =
    static member New (window: IWindow) (gl: GL) =
        match
            loadShaderFromManifestResources
                gl
                "Experiment.Assets.Shaders.shader.vert"
                "Experiment.Assets.Shaders.shader.frag"
        with
        | Error e -> Error e
        | Ok shader ->
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
                          uint32 2,
                          VertexAttributeSpecification.FromFieldName<Vertex>
                              4
                              VertexAttribPointerType.Float
                              false
                              "Color" ]
                        |> Map.ofList },
                    ReadOnlySpan
                        [| Vertex(new Vector2D<float32>(-0.5f, -0.5f), System.Drawing.Color.Red)
                           Vertex(new Vector2D<float32>(0.5f, -0.5f), System.Drawing.Color.Green)
                           Vertex(new Vector2D<float32>(0.5f, 0.5f), System.Drawing.Color.Blue)
                           Vertex(new Vector2D<float32>(-0.5f, 0.5f), System.Drawing.Color.Purple) |],
                    BufferUsageARB.DynamicDraw,
                    (ReadOnlySpan [| uint16 0; uint16 1; uint16 2; uint16 2; uint16 3; uint16 0 |]),
                    BufferUsageARB.DynamicDraw
                )

            Ok(new State(window, gl, shader, vertexArray))

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
            | Ok state -> state
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
