open Experiment
open Silk.NET.Windowing
open System
open Silk.NET.Maths
open Silk.NET.Input
open Silk.NET.WebGPU
open System.Runtime.InteropServices
open Microsoft.FSharp.NativeInterop

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
                let msg = Marshal.PtrToStringAnsi(NativePtr.toNativeInt msgPtr)
                result <- Some(Error(new Exception($"error getting adapter: {msg}"))))

    webgpu.InstanceRequestAdapter(instance, &options, callback, null)

    // 	webGPU.InstanceRequestAdapter(instance, ref options, callback, null);

    // 	if (error != null)
    // 	{
    // 		throw error;
    // 	}
    // 	if (result == null)
    // 	{
    // 		throw new Exception($"didn't create adapter, completed without callback being invoked");
    // 	}

    // 	var adapterProperties = new AdapterProperties();
    // 	webGPU.AdapterGetProperties(result, ref adapterProperties);
    // 	Console.WriteLine($"adapter type: {adapterProperties.AdapterType}");
    // 	Console.WriteLine($"adapter architecture: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.Architecture)}");
    // 	Console.WriteLine($"adapter backend type: {adapterProperties.BackendType}");
    // 	Console.WriteLine($"adapter device ID: {adapterProperties.DeviceID}");
    // 	Console.WriteLine($"adapter driver description: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.DriverDescription)}");
    // 	Console.WriteLine($"adapter name: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.Name)}");
    // 	Console.WriteLine($"adapter vendor ID: {adapterProperties.VendorID}");
    // 	Console.WriteLine($"adapter vendor name: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.VendorName)}");

    // 	return result;
    ()

type State private (window: IWindow) =
    static member New(window: IWindow) =
        let webgpu = WebGPU.GetApi()
        let instance = createInstance webgpu
        let surface = createSurface window webgpu instance
        let adapter = createAdapter webgpu instance surface
        // device = CreateDevice(webGPU, adapter);
        // surfaceTextureFormat = ConfigureSurface(webGPU, surface, device, window.Size);
        // ConfigureDebugCallback(webGPU, device);

        Ok(new State(window))

    interface IDisposable with
        member this.Dispose() : unit = ()

    member this.Resize(size: Vector2D<int>) = ()

    member this.Update(time: TimeSpan) = ()

    member this.Render() = ()

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
    state <-
        Some(
            match State.New window with
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
