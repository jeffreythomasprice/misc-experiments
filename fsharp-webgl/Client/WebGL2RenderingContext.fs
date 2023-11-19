namespace Client

open Microsoft.JSInterop
open System

type WebGL2RenderingContext(js: IJSInProcessRuntime, context: IJSUnmarshalledObjectReference) =
    member val COLOR_BUFFER_BIT = js.Invoke<int>("getValue", context, "COLOR_BUFFER_BIT")

    member val VERTEX_SHADER = js.Invoke<int>("getValue", context, "VERTEX_SHADER")
    member val FRAGMENT_SHADER = js.Invoke<int>("getValue", context, "FRAGMENT_SHADER")

    member val COMPILE_STATUS = js.Invoke<int>("getValue", context, "COMPILE_STATUS")
    member val LINK_STATUS = js.Invoke<int>("getValue", context, "LINK_STATUS")
    member val ACTIVE_ATTRIBUTES = js.Invoke<int>("getValue", context, "ACTIVE_ATTRIBUTES")
    member val ACTIVE_UNIFORMS = js.Invoke<int>("getValue", context, "ACTIVE_UNIFORMS")

    member val ARRAY_BUFFER = js.Invoke<int>("getValue", context, "ARRAY_BUFFER")

    member val STATIC_DRAW = js.Invoke<int>("getValue", context, "STATIC_DRAW")

    member this.arrayToFloat32Array(input: single array) =
        (js :?> IJSUnmarshalledRuntime)
            .InvokeUnmarshalled<float32 array, IJSUnmarshalledObjectReference>("arrayToFloat32Array", input)

    member this.clearColor (r: float) (g: float) (b: float) (a: float) =
        context.InvokeVoid("clearColor", r, g, b, a)

    member this.clear(bits: int) = context.InvokeVoid("clear", bits)

    member this.viewport (x: int) (y: int) (width: int) (height: int) =
        context.InvokeVoid("viewport", x, y, width, height)

    member this.createShader(typ: int) =
        context.Invoke<IJSInProcessObjectReference>("createShader", typ)

    member this.shaderSource (shader: IJSInProcessObjectReference) (source: string) =
        context.InvokeVoid("shaderSource", shader, source)

    member this.compileShader(shader: IJSInProcessObjectReference) =
        context.InvokeVoid("compileShader", shader)

    member this.getShaderParameter<'T> (shader: IJSInProcessObjectReference) (param: int) =
        context.Invoke<'T>("getShaderParameter", shader, param)

    member this.getShaderInfoLog(shader: IJSInProcessObjectReference) =
        context.Invoke<string>("getShaderInfoLog", shader)

    member this.deleteShader(shader: IJSInProcessObjectReference) =
        context.InvokeVoid("deleteShader", shader)

    member this.createProgram() =
        context.Invoke<IJSInProcessObjectReference>("createProgram")

    member this.attachShader (program: IJSInProcessObjectReference) (shader: IJSInProcessObjectReference) =
        context.InvokeVoid("attachShader", program, shader)

    member this.linkProgram(program: IJSInProcessObjectReference) =
        context.InvokeVoid("linkProgram", program)

    member this.getProgramParameter<'T> (program: IJSInProcessObjectReference) (param: int) =
        context.Invoke<'T>("getProgramParameter", program, param)

    member this.getProgramInfoLog(program: IJSInProcessObjectReference) =
        context.Invoke<string>("getProgramInfoLog", program)

    member this.deleteProgram(program: IJSInProcessObjectReference) =
        context.InvokeVoid("deleteProgram", program)

    member this.getActiveAttrib (program: IJSInProcessObjectReference) (index: int) =
        context.Invoke<IJSInProcessObjectReference>("getActiveAttrib", program, index)

    member this.getActiveUniform (program: IJSInProcessObjectReference) (index: int) =
        context.Invoke<IJSInProcessObjectReference>("getActiveUniform", program, index)

    member this.createBuffer() =
        context.Invoke<IJSInProcessObjectReference>("createBuffer")

    member this.bindBuffer (typ: int) (buffer: IJSInProcessObjectReference option) =
        context.InvokeVoid(
            "bindBuffer",
            typ,
            (match buffer with
             | Some(buffer) -> buffer
             | None -> null)
        )

    member this.bufferData (typ: int) (data: IJSUnmarshalledObjectReference) (usage: int) =
        context.InvokeVoid("bufferData", typ, data, usage)
