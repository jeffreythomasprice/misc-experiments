module Client.Main

open System
open Elmish
open Bolero
open Bolero.Html
open Microsoft.JSInterop
open Bolero.Remoting.Client

type ShaderActiveInfo =
    { index: int
      name: string
      size: int
      typ: int }

type Shader
    private
    (
        js: IJSInProcessRuntime,
        context: WebGL2RenderingContext,
        vertexShader: IJSInProcessObjectReference,
        fragmentShader: IJSInProcessObjectReference,
        program: IJSInProcessObjectReference
    ) =
    member val attributes =
        seq { 0 .. ((context.getProgramParameter program context.ACTIVE_ATTRIBUTES) - 1) }
        |> Seq.map (fun i ->
            let attrib = context.getActiveAttrib program i
            let name = js.Invoke<string>("getValue", attrib, "name")
            let size = js.Invoke<int>("getValue", attrib, "size")
            let typ = js.Invoke<int>("getValue", attrib, "type")

            let result: ShaderActiveInfo =
                { index = i
                  name = name
                  size = size
                  typ = typ }

            name, result)
        |> Map.ofSeq

    member val uniforms =
        seq { 0 .. ((context.getProgramParameter program context.ACTIVE_UNIFORMS) - 1) }
        |> Seq.map (fun i ->
            let attrib = context.getActiveUniform program i
            let name = js.Invoke<string>("getValue", attrib, "name")
            let size = js.Invoke<int>("getValue", attrib, "size")
            let typ = js.Invoke<int>("getValue", attrib, "type")

            let result: ShaderActiveInfo =
                { index = i
                  name = name
                  size = size
                  typ = typ }

            name, result)
        |> Map.ofSeq

    interface IDisposable with
        member this.Dispose() =
            context.deleteShader vertexShader
            context.deleteShader fragmentShader
            context.deleteProgram program

    static member create
        (js: IJSInProcessRuntime)
        (context: WebGL2RenderingContext)
        (vertexShaderSource: string)
        (fragmentShaderSource: string)
        =
        let createShader (typ: int) (source: string) =
            let result = context.createShader typ
            context.shaderSource result source
            context.compileShader result

            if not (context.getShaderParameter result context.COMPILE_STATUS) then
                let log = context.getShaderInfoLog result
                context.deleteShader result
                Error log
            else
                Ok result

        let vertexShader = createShader context.VERTEX_SHADER vertexShaderSource
        let fragmentShader = createShader context.FRAGMENT_SHADER fragmentShaderSource

        match (vertexShader, fragmentShader) with
        | Ok(vertexShader), Ok(fragmentShader) ->
            let program = context.createProgram ()
            context.attachShader program vertexShader
            context.attachShader program fragmentShader
            context.linkProgram program

            if not (context.getProgramParameter program context.LINK_STATUS) then
                context.deleteShader vertexShader
                context.deleteShader fragmentShader
                let log = context.getProgramInfoLog program
                context.deleteProgram program
                Error $"link error:\n{log}"
            else
                Ok(new Shader(js, context, vertexShader, fragmentShader, program))
        | Ok(vertexShader), Error(fragmentShader) ->
            context.deleteShader vertexShader
            Error $"compile error in fragment shader:\n{fragmentShader}"
        | Error(vertexShader), Ok(fragmentShader) ->
            context.deleteShader fragmentShader
            Error $"compile error in vertex shader:\n{vertexShader}"
        | Error(vertexShader), Error(fragmentShader) ->
            Error
                $"compile error in both vertex and fragment shaders\nvertex shader:\n{vertexShader}\nfragment shader:\n{fragmentShader}"

    member this.``use``() = context.useProgram (Some program)

type WebGLModel =
    { shader: Shader
      arrayBuffer: IJSInProcessObjectReference }

type Model = { webgl: WebGLModel option }

type Message =
    | Init
    | InitSuccess
    | ContextAvailable of context: WebGL2RenderingContext
    | Animate of context: WebGL2RenderingContext * time: float
    | Resize of context: WebGL2RenderingContext * width: int * height: int

let initModel = { webgl = None }, Cmd.ofMsg Init

let canvasRef = HtmlRef()

let update (js: IJSInProcessRuntime) message model =
    match message with
    | Init ->
        model,
        Cmd.OfTask.perform (fun _ -> task { do! js.InvokeVoidAsync("init", canvasRef.Value) }) () (fun _ -> InitSuccess)

    | InitSuccess -> model, Cmd.none

    | ContextAvailable(context) ->
        match
            Shader.create
                js
                context
                @"
                attribute vec2 positionAttribute;

                void main() {
                    gl_Position = vec4(positionAttribute, 0, 1);
                }
                "
                @"
                void main() {
                    gl_FragColor = vec4(1, 1, 1, 1);
                }
                "
        with
        | Ok(shader) ->
            printfn "TODO shader attributes = %A" shader.attributes
            printfn "TODO shader uniforms = %A" shader.uniforms

            let arrayBuffer = context.createBuffer ()
            context.bindBuffer context.ARRAY_BUFFER (Some arrayBuffer)

            context.bufferData
                context.ARRAY_BUFFER
                (context.createFloat32ArrayFromArray [| -0.5F; -0.5F; 0.5F; -0.5F; 0.0F; 0.5F |])
                context.STATIC_DRAW

            context.bindBuffer context.ARRAY_BUFFER None

            { webgl =
                Some
                    { shader = shader
                      arrayBuffer = arrayBuffer } },
            Cmd.none
        | Error(error) ->
            printfn "error making shader: %s" error
            model, Cmd.none

    | Animate(context, time) ->
        context.clearColor 0.25 0.5 0.75 1
        context.clear context.COLOR_BUFFER_BIT

        match model.webgl with
        | Some { shader = shader
                 arrayBuffer = arrayBuffer } ->
            shader.``use`` ()
            context.bindBuffer context.ARRAY_BUFFER (Some arrayBuffer)
            let positionAttribute = shader.attributes["positionAttribute"]
            context.vertexAttribPointer positionAttribute.index 2 context.FLOAT false 0 0
            context.enableVertexAttribArray positionAttribute.index
            context.drawArrays context.TRIANGLES 0 3
            context.disableVertexAttribArray positionAttribute.index
            context.bindBuffer context.ARRAY_BUFFER None
            context.useProgram None
        | _ -> ()

        model, Cmd.none

    | Resize(context, width, height) ->
        context.viewport 0 0 width height
        model, Cmd.none

let view model dispatch = canvas { canvasRef }

type App() =
    inherit ProgramComponent<Model, Message>()

    static let mutable instance: App option = None

    member val context: WebGL2RenderingContext option = None with get, set

    override this.Program =
        instance <- Some(this)
        Program.mkProgram (fun _ -> initModel) (update (this.JSRuntime :?> IJSInProcessRuntime)) view

    [<JSInvokable>]
    static member init(context: IJSUnmarshalledObjectReference) =
        match instance with
        | Some(instance) ->
            let context =
                WebGL2RenderingContext(instance.JSRuntime :?> IJSInProcessRuntime, context)

            instance.context <- Some(context)
            instance.Dispatch(ContextAvailable(context))
        | None -> ()

        ()

    [<JSInvokable>]
    static member animate(time: float) =
        match instance with
        | Some(instance) ->
            match instance.context with
            | Some(context) -> instance.Dispatch(Animate(context, time))
            | None -> ()
        | None -> ()

    [<JSInvokable>]
    static member resize (width: int) (height: int) =
        match instance with
        | Some(instance) ->
            match instance.context with
            | Some(context) -> instance.Dispatch(Resize(context, width, height))
            | None -> ()
        | None -> ()
