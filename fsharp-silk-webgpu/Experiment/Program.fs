open Experiment
open Silk.NET.Windowing
open System
open Silk.NET.Maths
open Silk.NET.Input
open Silk.NET.WebGPU
open System.Runtime.InteropServices

let createInstance (webgpu: WebGPU) =
    let descriptor = new InstanceDescriptor()
    let result = webgpu.CreateInstance &descriptor
    printfn "created instance"
    result

let createSurface (window: IWindow) (webgpu: WebGPU) (instance: nativeptr<Instance>) =
    let result = window.CreateWebGPUSurface(webgpu, instance)
    printfn "created surface"
    result

let createAdapter (webgpu: WebGPU) (instance: nativeptr<Instance>) (surface: nativeptr<Surface>) =
    let options =
        new RequestAdapterOptions(
            CompatibleSurface = surface,
            BackendType = BackendType.Vulkan,
            PowerPreference = PowerPreference.HighPerformance
        )

    let mutable result = None

    let callback =
        PfnRequestAdapterCallback.From(fun status adapter msgPtr userDataPtr ->
            if status = RequestAdapterStatus.Success then
                result <- Some(Ok adapter)
            else
                result <-
                    Some(
                        Error(
                            new Exception $"error getting adapter: {Marshalling.AnsiStringMarshaller.ConvertToManaged msgPtr}"
                        )
                    ))

    webgpu.InstanceRequestAdapter(instance, &options, callback, IntPtr.Zero.ToPointer())

    match result with
    | None -> Error(new Exception "expected adapter via callback by this point")
    | Some(Error e) -> Error e
    | Some(Ok result) ->
        let adapterProperties = new AdapterProperties()
        webgpu.AdapterGetProperties(result, ref adapterProperties)
        printfn $"adapter type: {adapterProperties.AdapterType}"

        printfn
            $"adapter architecture: {Marshalling.AnsiStringMarshaller.ConvertToManaged adapterProperties.Architecture}"

        printfn $"adapter backend type: {adapterProperties.BackendType}"
        printfn $"adapter device ID: {adapterProperties.DeviceID}"

        printfn
            $"adapter driver description: {Marshalling.AnsiStringMarshaller.ConvertToManaged adapterProperties.DriverDescription}"

        printfn $"adapter name: {Marshalling.AnsiStringMarshaller.ConvertToManaged adapterProperties.Name}"
        printfn $"adapter vendor ID: {adapterProperties.VendorID}"
        printfn $"adapter vendor name: {Marshalling.AnsiStringMarshaller.ConvertToManaged adapterProperties.VendorName}"

        Ok result

let createDevice (webgpu: WebGPU) (adapter: nativeptr<Adapter>) =
    let descriptor = new DeviceDescriptor()

    let mutable result = None

    let callback =
        PfnRequestDeviceCallback.From(fun status device msgPtr userDataPtr ->
            if status = RequestDeviceStatus.Success then
                result <- Some(Ok device)
            else
                result <-
                    Some(
                        Error(
                            new Exception $"error getting device: {Marshalling.AnsiStringMarshaller.ConvertToManaged msgPtr}"
                        )
                    ))

    webgpu.AdapterRequestDevice(adapter, &descriptor, callback, IntPtr.Zero.ToPointer())

    match result with
    | None -> Error(new Exception "expected adapter via callback by this point")
    | Some(Error e) -> Error e
    | Some(Ok result) ->
        printfn "created device"
        Ok result

let configureSurface
    (webgpu: WebGPU)
    (surface: nativeptr<Surface>)
    (device: nativeptr<Device>)
    (windowSize: Vector2D<int>)
    =
    let surfaceTextureFormat = TextureFormat.Bgra8Unorm

    let configuration =
        new SurfaceConfiguration(
            Device = device,
            Width = uint32 windowSize.X,
            Height = uint32 windowSize.Y,
            Format = surfaceTextureFormat,
            PresentMode = PresentMode.Fifo,
            Usage = TextureUsage.RenderAttachment
        )

    webgpu.SurfaceConfigure(surface, &configuration)

    surfaceTextureFormat

let configureDebugCallback (webgpu: WebGPU) (device: nativeptr<Device>) =
    let callback =
        PfnErrorCallback.From(fun errorType msgPtr userDataPtr ->
            printfn "unhandled WebGPU error: {Marshalling.AnsiStringMarshaller.ConvertToManaged msgPtr}")

    webgpu.DeviceSetUncapturedErrorCallback(device, callback, IntPtr.Zero.ToPointer())

type State private (window: IWindow, webgpu: WebGPU, surface: nativeptr<Surface>, device: nativeptr<Device>) =
    static member New(window: IWindow) =
        let webgpu = WebGPU.GetApi()
        let instance = createInstance webgpu
        let surface = createSurface window webgpu instance

        createAdapter webgpu instance surface
        |> Result.bind (fun adapter -> createDevice webgpu adapter)
        |> Result.map (fun device ->
            let surfaceTextureFormat = configureSurface webgpu surface device window.Size

            configureDebugCallback webgpu device

            new State(window, webgpu, surface, device))

    interface IDisposable with
        member this.Dispose() : unit = ()

    member this.Resize(size: Vector2D<int>) = ()

    member this.Update(time: TimeSpan) = ()

    member this.Render() =
        // TODO wrap in dispsoables so we can 'use' them?
        let commandEncoder =
            webgpu.DeviceCreateCommandEncoder(device, Span<CommandEncoderDescriptor>.Empty)

        let mutable surfaceTexture = SurfaceTexture()
        webgpu.SurfaceGetCurrentTexture(surface, &surfaceTexture)

        let surfaceTextureView =
            webgpu.TextureCreateView(surfaceTexture.Texture, Span<TextureViewDescriptor>.Empty)

        let mutable colorAttachments: RenderPassColorAttachment nativeptr =
            Microsoft.FSharp.NativeInterop.NativePtr.stackalloc 1

        let colorAttachmentsSpan =
            Span<RenderPassColorAttachment>(colorAttachments |> NativeInterop.NativePtr.toVoidPtr, 1)

        colorAttachmentsSpan[0] <-
            new RenderPassColorAttachment(
                View = surfaceTextureView,
                LoadOp = LoadOp.Clear,
                ClearValue = new Color(0.25, 0.5, 1.0, 1.0),
                StoreOp = StoreOp.Store
            )

        let renderPassDescriptor =
            new RenderPassDescriptor(colorAttachmentCount = unativeint 1, colorAttachments = colorAttachments)

        let renderPassEncoder =
            webgpu.CommandEncoderBeginRenderPass(commandEncoder, &renderPassDescriptor)

        // TODO do some actual drawing

        webgpu.RenderPassEncoderEnd renderPassEncoder

        let commandBuffer =
            webgpu.CommandEncoderFinish(commandEncoder, Span<CommandBufferDescriptor>.Empty)

        let queue = webgpu.DeviceGetQueue device
        webgpu.QueueSubmit(queue, unativeint 1, ref commandBuffer)

        webgpu.SurfacePresent surface

        webgpu.CommandBufferRelease commandBuffer
        webgpu.RenderPassEncoderRelease renderPassEncoder
        webgpu.TextureViewRelease surfaceTextureView
        webgpu.TextureRelease surfaceTexture.Texture
        webgpu.CommandEncoderRelease commandEncoder

    member this.KeyDown(key: Key) = ()

    member this.KeyUp(key: Key) =
        match key with
        | Key.Escape -> window.Close()
        | _ -> ()

let window =
    Window.Create(new WindowOptions(Title = "Experiment", Size = new Vector2D<int>(1024, 768)))

let mutable state = None

window.add_Load (fun () ->
    state <-
        Some(
            match State.New window with
            | Ok state ->
                state.Resize window.Size
                state
            | Error e -> failwith (sprintf "error initializing: %A" e)
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
