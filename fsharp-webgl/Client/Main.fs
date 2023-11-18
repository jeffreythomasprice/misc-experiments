module Client.Main

open Elmish
open Bolero
open Bolero.Html
open Microsoft.JSInterop
open Bolero.Remoting.Client

type WebGL2RenderingContext(js: IJSInProcessRuntime, context: IJSInProcessObjectReference) =
    let context = context

    member val COLOR_BUFFER_BIT = js.Invoke<int>("getValue", context, "COLOR_BUFFER_BIT")

    member this.clearColor (r: float) (g: float) (b: float) (a: float) =
        context.InvokeVoid("clearColor", r, g, b, a)

    member this.clear(bits: int) = context.InvokeVoid("clear", bits)

    member this.viewport (x: int) (y: int) (width: int) (height: int) =
        context.InvokeVoid("viewport", x, y, width, height)

type Model = unit

type Message =
    | Init
    | InitSuccess
    | Animate of context: WebGL2RenderingContext * time: float
    | Resize of context: WebGL2RenderingContext * width: int * height: int

let initModel = (), Cmd.ofMsg Init

let canvasRef = HtmlRef()

let update (js: IJSRuntime) message model =
    match message with
    | Init ->
        model,
        Cmd.OfTask.perform (fun _ -> task { do! js.InvokeVoidAsync("init", canvasRef.Value) }) () (fun _ -> InitSuccess)
    | InitSuccess -> model, Cmd.none
    | Animate(context, time) ->
        context.clearColor 0.25 0.5 0.75 1
        context.clear context.COLOR_BUFFER_BIT
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
        Program.mkProgram (fun _ -> initModel) (update this.JSRuntime) view

    [<JSInvokable>]
    static member init(context: IJSInProcessObjectReference) =
        match instance with
        | Some(instance) ->
            instance.context <- Some(WebGL2RenderingContext(instance.JSRuntime :?> IJSInProcessRuntime, context))
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
