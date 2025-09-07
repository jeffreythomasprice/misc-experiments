import CLib
import CSDL
import CWGPU
import Foundation

extension String {
	public static func fromWGPUStringView(other: WGPUStringView) -> String {
		let buflen = other.length + 1
		let buf = UnsafeMutablePointer<CChar>.allocate(capacity: buflen)
		defer {
			buf.deallocate()
		}
		if other.length > 0 {
			memcpy(buf, other.data, other.length)
		}
		buf[other.length] = 0
		return String(cString: buf)
	}

	public func toWGPUStringView() -> WGPUStringView {
		let utf8 = self.utf8
		let count = utf8.count
		let buf = UnsafeMutablePointer<CChar>.allocate(capacity: count)
		var index = 0
		for byte in utf8 {
			buf[index] = CChar(byte)
			index += 1
		}
		return WGPUStringView(data: buf, length: count)
	}
}

@MainActor
func createWGPUInstance() -> WGPUInstance {
	let result = wgpuCreateInstance(nil)!
	print("created wgpu instance")
	return result
}

enum CreateWGPUSurfaceError: Error {
	case FailedToGetWindow
	case FailedToGetWindowsModule
	case FailedToGetDisplay
	case UnsupportedPlatform
}

@MainActor
func createWGPUSurface(sdlWindow: OpaquePointer, wgpuInstance: WGPUInstance) -> Result<
	WGPUSurface, CreateWGPUSurfaceError
> {
	// implement for other platforms
	// https://github.com/eliemichel/sdl3webgpu/blob/main/sdl3webgpu.c

	let props = SDL_GetWindowProperties(sdlWindow)

	#if os(Windows)
		guard let hwnd = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WIN32_HWND_POINTER, nil)
		else {
			print("failed to get win32 hwnd")
			return .failure(.FailedToGetWindow)
		}

		guard let hInstance = GetModuleHandleA(nil) else {
			print("failed to get hinstance")
			return .failure(.FailedToGetWindowsModule)
		}

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
	#else
		#if os(Linux)
			let platform = String(cString: SDL_GetCurrentVideoDriver())
			print("video platform: \(platform)")
			let result: WGPUSurface?

			switch platform {
			case "x11":
				guard
					let x11Display = SDL_GetPointerProperty(
						props, SDL_PROP_WINDOW_X11_DISPLAY_POINTER, nil)
				else {
					print("failed to get x11 display")
					return .failure(.FailedToGetDisplay)
				}
				let x11Window = SDL_GetNumberProperty(
					props, SDL_PROP_WINDOW_X11_WINDOW_NUMBER, 0)
				guard x11Window != 0 else {
					print("failed to get x11 window")
					return .failure(.FailedToGetWindow)
				}
				var fromXLibWindow = WGPUSurfaceSourceXlibWindow()
				fromXLibWindow.chain.sType = WGPUSType_SurfaceSourceXlibWindow
				fromXLibWindow.chain.next = nil
				fromXLibWindow.display = x11Display
				fromXLibWindow.window = UInt64(x11Window)

				result = withUnsafePointer(to: &fromXLibWindow.chain) {
					fromXLibWindowChainPtr in
					var surfaceDescriptor = WGPUSurfaceDescriptor()
					surfaceDescriptor.nextInChain = fromXLibWindowChainPtr
					surfaceDescriptor.label = WGPUStringView(data: nil, length: 0)
					return wgpuInstanceCreateSurface(wgpuInstance, &surfaceDescriptor)
				}
			case "wayland":
				print("TODO support wayland")
				/*
				void *wayland_display = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER, NULL);
				void *wayland_surface = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WAYLAND_SURFACE_POINTER, NULL);
				if (!wayland_display || !wayland_surface) return NULL;
				
				WGPUSurfaceSourceWaylandSurface fromWaylandSurface;
				fromWaylandSurface.chain.sType = WGPUSType_SurfaceSourceWaylandSurface;
				fromWaylandSurface.chain.next = NULL;
				fromWaylandSurface.display = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_WAYLAND_DISPLAY_POINTER, NULL);
				fromWaylandSurface.surface = wayland_surface;
				
				WGPUSurfaceDescriptor surfaceDescriptor;
				surfaceDescriptor.nextInChain = &fromWaylandSurface.chain;
				surfaceDescriptor.label = (WGPUStringView){ NULL, WGPU_STRLEN };
				
				return wgpuInstanceCreateSurface(instance, &surfaceDescriptor);
				*/
				result = nil
			default:
				print("unsupported video platform: \(platform)")
				return .failure(.UnsupportedPlatform)
			}
		#endif
	#endif

	guard let result = result else {
		print("failed to create wgpu surface")
		return .failure(.FailedToGetWindow)
	}
	return .success(result)
}

enum CreateWGPUAdapterError: Error {
	case FailedToCreateAdapter
	case FailedToGetAdapterInfo
}

@MainActor
func createWGPUAdapter(wgpuInstance: WGPUInstance, wgpuSurface: WGPUSurface) -> Result<
	WGPUAdapter, CreateWGPUAdapterError
> {
	var adapterOptions = WGPURequestAdapterOptions()
	adapterOptions.powerPreference = WGPUPowerPreference_HighPerformance
	adapterOptions.backendType = WGPUBackendType_Vulkan
	adapterOptions.compatibleSurface = wgpuSurface

	var callbackInfo = WGPURequestAdapterCallbackInfo()
	var result: WGPUAdapter? = nil
	withUnsafeMutablePointer(to: &result) {
		callbackInfo.userdata1 = UnsafeMutableRawPointer($0)
		callbackInfo.callback = { status, adapter, message, userdata1, userdata2 in
			let message = String.fromWGPUStringView(other: message)
			if !message.isEmpty {
				print("adapter callback: \(message)")
			}
			UnsafeMutablePointer<WGPUAdapter?>(.init(userdata1))?.pointee = adapter
		}

		// trying to wait for future results in rust not implemented error
		wgpuInstanceRequestAdapter(wgpuInstance, &adapterOptions, callbackInfo)
		// let future = wgpuInstanceRequestAdapter(wgpuInstance, &adapterOptions, callbackInfo)
		// var futureWaitInfo = [WGPUFutureWaitInfo(future: future, completed: 0)]
		// assert(wgpuInstanceWaitAny(wgpuInstance, 1, &futureWaitInfo, 5000) == WGPUWaitStatus_Success)
	}

	guard let result = result else {
		print("created wgpu adapter")
		return .failure(.FailedToCreateAdapter)
	}

	var adapterInfo = WGPUAdapterInfo()
	guard wgpuAdapterGetInfo(result, &adapterInfo) == WGPUStatus_Success else {
		print("failed to get adapter info")
		return .failure(.FailedToGetAdapterInfo)
	}
	print("adapter type: \(adapterInfo.adapterType)")
	print("adapter architecture: \(String.fromWGPUStringView(other: adapterInfo.architecture))")
	print("adapter backend type: \(adapterInfo.backendType)")
	print("adapter device ID: \(adapterInfo.deviceID)")
	print(
		"adapter driver description: \(String.fromWGPUStringView(other: adapterInfo.description))")
	print("adapter vendor ID: \(adapterInfo.vendorID)")
	print("adapter vendor name: \(String.fromWGPUStringView(other: adapterInfo.vendor))")

	return .success(result)
}

enum CreateWGPUDeviceError: Error {
	case FailedToCreateDevice
}

func createWGPUDevice(wgpuAdapter: WGPUAdapter) -> Result<WGPUDevice, CreateWGPUDeviceError> {
	var callbackInfo = WGPURequestDeviceCallbackInfo()
	var result: WGPUDevice? = nil
	withUnsafeMutablePointer(to: &result) {
		callbackInfo.userdata1 = UnsafeMutableRawPointer($0)
		callbackInfo.callback = { status, device, message, userdata1, userdata2 in
			let message = String.fromWGPUStringView(other: message)
			if !message.isEmpty {
				print("device callback: \(message)")
			}
			UnsafeMutablePointer<WGPUDevice?>(.init(userdata1))?.pointee = device
		}

		// trying to wait for future results in rust not implemented error
		wgpuAdapterRequestDevice(wgpuAdapter, nil, callbackInfo)
		// let future = wgpuAdapterRequestDevice(wgpuAdapter, &deviceDescriptor, callbackInfo)
		// var futureWaitInfo = [WGPUFutureWaitInfo(future: future, completed: 0)]
		// assert(wgpuInstanceWaitAny(wgpuInstance, 1, &futureWaitInfo, 5000) == WGPUWaitStatus_Success)
	}

	guard let result = result else {
		print("failed to create wgpu device")
		return .failure(.FailedToCreateDevice)
	}
	print("created wgpu device")
	return .success(result)
}

enum CreateWGPUShaderModuleError: Error {
	case FailedToCreateShaderModule
}

func createWGPUShaderModuleWGSL(wgpuDevice: WGPUDevice, shaderSource: String) -> Result<
	WGPUShaderModule, CreateWGPUShaderModuleError
> {
	var wgpuShaderSource = WGPUShaderSourceWGSL(
		chain: WGPUChainedStruct(next: nil, sType: WGPUSType_ShaderSourceWGSL),
		code: shaderSource.toWGPUStringView())
	let result = withUnsafePointer(to: &wgpuShaderSource.chain) { wgpuShaderSourcePtr in
		var shaderModuleDescriptor = WGPUShaderModuleDescriptor()
		shaderModuleDescriptor.nextInChain = wgpuShaderSourcePtr
		return wgpuDeviceCreateShaderModule(wgpuDevice, &shaderModuleDescriptor)
	}
	guard let result = result else {
		print("failed to create wgpu shader module")
		return .failure(.FailedToCreateShaderModule)
	}
	print("created wgpu shader module")
	return .success(result)
}

enum CreateWGPUQueueError: Error {
	case FailedToCreateQueue
}

func createWGPUQueue(wgpuDevice: WGPUDevice) -> Result<WGPUQueue, CreateWGPUQueueError> {
	let result = wgpuDeviceGetQueue(wgpuDevice)
	guard let result = result else {
		print("failed to create wgpu queue")
		return .failure(.FailedToCreateQueue)
	}
	print("created wgpu queue")
	return .success(result)
}

enum GetWGPUSurfaceCapabilitiesError: Error {
	case FailedToGetSurfaceCapabilities
}

func getWGPUSurfaceCapabilities(wgpuSurface: WGPUSurface, wgpuAdapter: WGPUAdapter)
	-> Result<WGPUSurfaceCapabilities, GetWGPUSurfaceCapabilitiesError>
{
	var surfaceCapabilities = WGPUSurfaceCapabilities()
	let status = wgpuSurfaceGetCapabilities(wgpuSurface, wgpuAdapter, &surfaceCapabilities)
	guard status == WGPUStatus_Success else {
		print("failed to get wgpu surface capabilities, status: \(status)")
		return .failure(.FailedToGetSurfaceCapabilities)
	}
	print("got wgpu surface capabilities")
	return .success(surfaceCapabilities)
}

enum CreateWGPUPipelineLayoutError: Error {
	case FailedToCreatePipelineLayout
}

func createWGPUPipelineLayout(wgpuDevice: WGPUDevice) -> Result<
	WGPUPipelineLayout, CreateWGPUPipelineLayoutError
> {
	let descriptor = WGPUPipelineLayoutDescriptor()
	let result = withUnsafePointer(to: descriptor) { descriptorPtr in
		wgpuDeviceCreatePipelineLayout(wgpuDevice, descriptorPtr)
	}
	guard let result = result else {
		print("failed to create wgpu pipeline layout")
		return .failure(.FailedToCreatePipelineLayout)
	}
	print("created wgpu pipeline layout")
	return .success(result)
}

enum CreateWGPURenderPipelineError: Error {
	case FailedToCreateRenderPipeline
}

func createWGPURenderPipeline(
	wgpuDevice: WGPUDevice, wgpuPipelineLayout: WGPUPipelineLayout,
	wgpuShaderModule: WGPUShaderModule,
	colorTargetFormat: WGPUTextureFormat
)
	-> Result<WGPURenderPipeline, CreateWGPURenderPipelineError>
{
	var descriptor = WGPURenderPipelineDescriptor()

	descriptor.layout = wgpuPipelineLayout

	descriptor.vertex.module = wgpuShaderModule
	descriptor.vertex.entryPoint = "vs_main".toWGPUStringView()
	let vertexAttributes = [
		// TODO something smarter for how vertex attributes work
		WGPUVertexAttribute(
			format: WGPUVertexFormat_Float32x2, offset: 0, shaderLocation: 0),
		WGPUVertexAttribute(
			format: WGPUVertexFormat_Float32x4, offset: 2 * 4, shaderLocation: 1),
	]
	var vertexBufferLayout = WGPUVertexBufferLayout()
	vertexBufferLayout.stepMode = WGPUVertexStepMode_Vertex
	// TODO something smarter for how vertex attributes work
	vertexBufferLayout.arrayStride = (2 + 4) * 4
	vertexBufferLayout.attributeCount = vertexAttributes.count
	vertexAttributes.withUnsafeBufferPointer { bufferPtr in
		vertexBufferLayout.attributes = bufferPtr.baseAddress
		withUnsafePointer(to: &vertexBufferLayout) { vertexBufferLayoutPtr in
			descriptor.vertex.bufferCount = 1
			descriptor.vertex.buffers = vertexBufferLayoutPtr
		}
	}

	var fragment = WGPUFragmentState()
	fragment.module = wgpuShaderModule
	fragment.entryPoint = "fs_main".toWGPUStringView()
	var colorTargetState = WGPUColorTargetState()
	colorTargetState.format = colorTargetFormat
	colorTargetState.writeMask = WGPUColorWriteMask_All
	let fragmentTargets = [colorTargetState]
	fragment.targetCount = fragmentTargets.count
	fragmentTargets.withUnsafeBufferPointer { bufferPtr in
		fragment.targets = bufferPtr.baseAddress
		withUnsafePointer(to: &fragment) { fragmentPtr in
			descriptor.fragment = fragmentPtr
		}
	}

	descriptor.primitive.topology = WGPUPrimitiveTopology_TriangleList

	descriptor.multisample.count = 1
	descriptor.multisample.mask = 0xFFFF_FFFF

	guard let result = wgpuDeviceCreateRenderPipeline(wgpuDevice, &descriptor) else {
		print("failed to create wgpu render pipeline")
		return .failure(.FailedToCreateRenderPipeline)
	}
	print("created wgpu render pipeline")
	return .success(result)
}

func createWGPUSurfaceConfiguration(
	wgpuDevice: WGPUDevice, textureFormat: WGPUTextureFormat, alphaMode: WGPUCompositeAlphaMode
) -> WGPUSurfaceConfiguration {
	var result = WGPUSurfaceConfiguration()
	result.device = wgpuDevice
	result.usage = WGPUTextureUsage_RenderAttachment
	result.format = textureFormat
	result.presentMode = WGPUPresentMode_Fifo
	result.alphaMode = alphaMode
	return result
}

func resizeWGPUSurfaceConfiguration(
	wgpuSurface: inout WGPUSurface,
	wgpuSurfaceConfiguration: inout WGPUSurfaceConfiguration, sdlWindow: OpaquePointer
) {
	var width: Int32 = 0
	var height: Int32 = 0
	SDL_GetWindowSize(sdlWindow, &width, &height)
	wgpuSurfaceConfiguration.width = UInt32(width)
	wgpuSurfaceConfiguration.height = UInt32(height)
	wgpuSurfaceConfigure(wgpuSurface, &wgpuSurfaceConfiguration)
	print("resized wgpu surface configuration to \(width)x\(height)")
}

print("TODO add result = \(add(1,2))")

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
let SDL_WINDOW_RESIZABLE = SDL_WindowFlags(0x0000_0000_0000_0020)
let sdlWindow = SDL_CreateWindow("Experiment", 1024, 768, SDL_WINDOW_VULKAN | SDL_WINDOW_RESIZABLE)
guard sdlWindow != nil else {
	let error = String(cString: SDL_GetError())
	print("failed to create SDL window: \(error)")
	exit(1)
}
defer {
	SDL_DestroyWindow(sdlWindow)
}

let wgpuInstance = createWGPUInstance()
var wgpuSurface = try createWGPUSurface(sdlWindow: sdlWindow!, wgpuInstance: wgpuInstance).get()
let wgpuAdapter = try createWGPUAdapter(wgpuInstance: wgpuInstance, wgpuSurface: wgpuSurface).get()
let wgpuDevice = try createWGPUDevice(wgpuAdapter: wgpuAdapter).get()
let wgpuQueue = try createWGPUQueue(wgpuDevice: wgpuDevice).get()

let wgpuShader = try createWGPUShaderModuleWGSL(
	wgpuDevice: wgpuDevice,
	shaderSource: String(decoding: Data(PackageResources.shader_wsgl), as: UTF8.self)
).get()

let wgpuSurfaceCapabilities = try getWGPUSurfaceCapabilities(
	wgpuSurface: wgpuSurface, wgpuAdapter: wgpuAdapter
).get()
let surfaceTextureFormat = wgpuSurfaceCapabilities.formats[0]

let wgpuPipelineLayout = try createWGPUPipelineLayout(wgpuDevice: wgpuDevice).get()
let wgpuRenderPipeline = try createWGPURenderPipeline(
	wgpuDevice: wgpuDevice, wgpuPipelineLayout: wgpuPipelineLayout,
	wgpuShaderModule: wgpuShader, colorTargetFormat: surfaceTextureFormat
).get()

var wgpuSurfaceConfig = createWGPUSurfaceConfiguration(
	wgpuDevice: wgpuDevice, textureFormat: surfaceTextureFormat,
	alphaMode: wgpuSurfaceCapabilities.alphaModes[0]
)
resizeWGPUSurfaceConfiguration(
	wgpuSurface: &wgpuSurface, wgpuSurfaceConfiguration: &wgpuSurfaceConfig, sdlWindow: sdlWindow!)

let framesPerSecond = 60
let delayBetweenFrames = 1000 / framesPerSecond

var exiting = false
while !exiting {
	var e = SDL_Event.init()
	if SDL_PollEvent(&e) {
		switch SDL_EventType(rawValue: e.type) {
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
		case SDL_EVENT_WINDOW_RESIZED:
			resizeWGPUSurfaceConfiguration(
				wgpuSurface: &wgpuSurface, wgpuSurfaceConfiguration: &wgpuSurfaceConfig,
				sdlWindow: sdlWindow!)
		default:
			break
		}
	}

	let wgpuCommandEncoder = wgpuDeviceCreateCommandEncoder(wgpuDevice, nil)
	// TODO no assert
	assert(wgpuCommandEncoder != nil)

	// TODO more wgpu stuff
	// https://github.com/gfx-rs/wgpu-native/blob/trunk/examples/triangle/main.c

	wgpuCommandEncoderRelease(wgpuCommandEncoder)

	SDL_Delay(Uint32(delayBetweenFrames))
}
