import CLib
import CSDL
import CWGPU

extension String {
	public static func fromWGPUStringView(other: WGPUStringView) -> String {
		let buflen = other.length + 1
		let buf = UnsafeMutablePointer<CChar>.allocate(capacity: buflen)
		defer {
			buf.deallocate()
		}
		memcpy(buf, other.data, other.length)
		buf[other.length] = 0
		return String(cString: buf)
	}
}

@MainActor
func createWGPUInstance() -> WGPUInstance {
	let result = wgpuCreateInstance(nil)!
	print("created wgpu instance")
	return result
}

@MainActor
func createWGPUSurface(sdlWindow: OpaquePointer, wgpuInstance: WGPUInstance) -> WGPUSurface {
	// implement for other platforms, not just windows
	// https://github.com/eliemichel/sdl3webgpu/blob/main/sdl3webgpu.c

	let props = SDL_GetWindowProperties(window)

	let hwnd = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WIN32_HWND_POINTER, nil)
	assert(hwnd != nil)

	let hInstance = GetModuleHandleA(nil)
	assert(hInstance != nil)

	var fromWindowsHWND = WGPUSurfaceSourceWindowsHWND()
	fromWindowsHWND.chain.sType = WGPUSType_SurfaceSourceWindowsHWND
	fromWindowsHWND.chain.next = nil
	fromWindowsHWND.hinstance = UnsafeMutableRawPointer(hInstance!)
	fromWindowsHWND.hwnd = hwnd
	let result = withUnsafePointer(to: &fromWindowsHWND.chain) { fromWindowsHWNDChainPtr in
		var surfaceDescriptor = WGPUSurfaceDescriptor()
		surfaceDescriptor.nextInChain = fromWindowsHWNDChainPtr
		surfaceDescriptor.label = WGPUStringView(data: nil, length: 0)
		return wgpuInstanceCreateSurface(wgpuInstance, &surfaceDescriptor)
	}
	assert(result != nil)
	print("created wgpu surface")
	return result!
}

@MainActor
func createWGPUAdapter(wgpuInstance: WGPUInstance, wgpuSurface: WGPUSurface) -> WGPUAdapter {
	var adapterOptions = WGPURequestAdapterOptions()
	adapterOptions.powerPreference = WGPUPowerPreference_HighPerformance
	adapterOptions.backendType = WGPUBackendType_Vulkan
	adapterOptions.compatibleSurface = wgpuSurface
	var callbackInfo = WGPURequestAdapterCallbackInfo()
	var result: WGPUAdapter? = nil
	callbackInfo.callback = { status, adapter, message, userdata1, userdata2 in
		print("adapter callback: ", String.fromWGPUStringView(other: message))
		result = adapter
	}

	// trying to wait for future results in rust not implemented error
	wgpuInstanceRequestAdapter(wgpuInstance, &adapterOptions, callbackInfo)
	// let future = wgpuInstanceRequestAdapter(wgpuInstance, &adapterOptions, callbackInfo)
	// var futureWaitInfo = [WGPUFutureWaitInfo(future: future, completed: 0)]
	// assert(wgpuInstanceWaitAny(wgpuInstance, 1, &futureWaitInfo, 5000) == WGPUWaitStatus_Success)

	assert(result != nil)

	// TODO print adapter info

	return result!
}

print("add result = \(add(1,2))")

if !SDL_Init(SDL_INIT_VIDEO) {
	let error = String(cString: SDL_GetError())
	print("SDL init error: \(error)")
	exit(1)
}
defer {
	SDL_Quit()
}

print("SDL version = \(SDL_GetVersion())")

// let SDL_WINDOW_OPENGL = SDL_WindowFlags(0x0000_0000_0000_0002)
let SDL_WINDOW_VULKAN = SDL_WindowFlags(0x0000_0000_1000_0000)
let window = SDL_CreateWindow("Experiment", 1024, 768, SDL_WINDOW_VULKAN)
if window == nil {
	let error = String(cString: SDL_GetError())
	print("failed to create SDL window: \(error)")
	exit(1)
}
defer {
	SDL_DestroyWindow(window)
}

let wgpuInstance = createWGPUInstance()
let wgpuSurface = createWGPUSurface(sdlWindow: window!, wgpuInstance: wgpuInstance)
let wgpuAdapter = createWGPUAdapter(wgpuInstance: wgpuInstance, wgpuSurface: wgpuSurface)
// TODO rest of wgpu stuff, device, etc.
// https://github.com/gfx-rs/wgpu-native/blob/trunk/examples/triangle/main.c

let framesPerSecond = 60
let delayBetweenFrames = 1000 / framesPerSecond

var exiting = false
while !exiting {
	var e = SDL_Event.init()
	if SDL_PollEvent(&e) {
		switch SDL_EventType(rawValue: Int32(e.type)) {
		case SDL_EVENT_QUIT:
			exiting = true
			break
		case SDL_EVENT_KEY_UP:
			switch e.key.key {
			case SDLK_ESCAPE:
				exiting = true
				break
			default:
				break
			}
			break
		default:
			break
		}
	}

	// TODO do some wgpu stuff
	// glClearColor(0.25, 0.5, 1, 1)
	// glClear(GLbitfield(GL_COLOR_BUFFER_BIT))
	// SDL_GL_SwapWindow(window)

	SDL_Delay(Uint32(delayBetweenFrames))
}
