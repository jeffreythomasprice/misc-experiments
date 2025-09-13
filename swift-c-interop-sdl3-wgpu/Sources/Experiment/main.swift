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

struct Vector2<T> {
	var x: T
	var y: T
}

struct RGBA<T> {
	var r: T
	var g: T
	var b: T
	var a: T
}

struct Vertex {
	var position: Vector2<Float>
	var color: RGBA<Float>
}

extension Array {
	mutating func asData<ResultType>(_ body: (Data?) -> ResultType) -> ResultType {
		self.withUnsafeMutableBytes {
			let data: Data? =
				if let ptr = $0.baseAddress {
					Data(
						bytesNoCopy: UnsafeMutableRawPointer(ptr), count: $0.count,
						deallocator: Data.Deallocator.none)
				} else {
					nil
				}
			return body(data)
		}
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
	return fragmentTargets.withUnsafeBufferPointer { bufferPtr in
		fragment.targets = bufferPtr.baseAddress
		return withUnsafePointer(to: &fragment) { fragmentPtr in
			descriptor.fragment = fragmentPtr

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
	}
}

func createWGPUSurfaceConfiguration(
	wgpuDevice: WGPUDevice, textureFormat: WGPUTextureFormat, alphaMode: WGPUCompositeAlphaMode
) -> WGPUSurfaceConfiguration {
	var result = WGPUSurfaceConfiguration()
	result.device = wgpuDevice
	result.usage = WGPUTextureUsage_RenderAttachment
	result.format = textureFormat
	result.presentMode = WGPUPresentMode_Mailbox
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

enum CreateWGPUTextureViewError: Error {
	case FailedToCreateTextureView
}

func createWGPUTextureView(wgpuTexture: WGPUTexture) -> Result<
	WGPUTextureView, CreateWGPUTextureViewError
> {
	guard let result = wgpuTextureCreateView(wgpuTexture, nil) else {
		print("failed to create wgpu texture view")
		return .failure(.FailedToCreateTextureView)
	}
	return .success(result)
}

enum CreateWGPUCommandEncoderError: Error {
	case FailedToCreateCommandEncoder
}

func createWGPUCommandEncoder(wgpuDevice: WGPUDevice, label: String? = nil) -> Result<
	WGPUCommandEncoder, CreateWGPUCommandEncoderError
> {
	var commandEncoderDescriptor = WGPUCommandEncoderDescriptor()
	if let label = label {
		commandEncoderDescriptor.label = label.toWGPUStringView()
	}
	guard let commandEncoder = wgpuDeviceCreateCommandEncoder(wgpuDevice, &commandEncoderDescriptor)
	else {
		print("failed to create command encoder")
		return .failure(.FailedToCreateCommandEncoder)
	}
	return .success(commandEncoder)
}

enum CreateWGPURenderPassEncoderError: Error {
	case FailedToCreateRenderPassEncoder
}

func createWGPURenderPassEncoder(
	wgpuCommandEncoder: WGPUCommandEncoder,
	wgpuRenderColorPassAttachements: [WGPURenderPassColorAttachment]
) -> Result<WGPURenderPassEncoder, CreateWGPURenderPassEncoderError> {
	var renderPassEncoderDescription = WGPURenderPassDescriptor()
	renderPassEncoderDescription.colorAttachmentCount = wgpuRenderColorPassAttachements.count
	let renderPassEncoder = wgpuRenderColorPassAttachements.withUnsafeBufferPointer {
		colorAttachmentsPtr in
		renderPassEncoderDescription.colorAttachments = colorAttachmentsPtr.baseAddress
		return wgpuCommandEncoderBeginRenderPass(wgpuCommandEncoder, &renderPassEncoderDescription)
	}
	guard let renderPassEncoder = renderPassEncoder else {
		print("failed to create render pass encoder")
		return .failure(.FailedToCreateRenderPassEncoder)
	}
	return .success(renderPassEncoder)
}

enum DoWGPUCommandEncoderFinishError: Error {
	case FailedToFinishCommandEncoder
}

func doWGPUCommandEncoderFinish(wgpuCommandEncoder: WGPUCommandEncoder) -> Result<
	WGPUCommandBuffer, DoWGPUCommandEncoderFinishError
> {
	let result = wgpuCommandEncoderFinish(wgpuCommandEncoder, nil)
	guard let result = result else {
		print("failed to finish command encoder")
		return .failure(.FailedToFinishCommandEncoder)
	}
	return .success(result)
}

func doWGPUQueueSubmit(wgpuQueue: WGPUQueue, wgpuCommandBuffers: [WGPUCommandBuffer?]) {
	wgpuCommandBuffers.withUnsafeBufferPointer { wgpuCommandBuffersPtr in
		wgpuQueueSubmit(wgpuQueue, wgpuCommandBuffers.count, wgpuCommandBuffersPtr.baseAddress)
	}
}

enum CreateWGPUBufferInitError: Error {
	case FailedToCreateBuffer
	case FailedToMapBufferMemory
	case FailedToGetContentBaseAddress
}

func createWGPUBufferInit<T>(
	wgpuDevice: WGPUDevice, label: String? = nil, content: inout [T], usage: WGPUBufferUsage
) -> Result<WGPUBuffer, CreateWGPUBufferInitError> {
	var descriptor = WGPUBufferDescriptor()
	if let label = label {
		descriptor.label = label.toWGPUStringView()
	}
	descriptor.usage = usage
	return content.asData { contentData in
		let contentCount = contentData?.count ?? 0
		descriptor.size = UInt64(contentCount)

		// true if we have at least one byte to copy
		descriptor.mappedAtCreation = if contentCount > 0 { 1 } else { 0 }

		guard let result = wgpuDeviceCreateBuffer(wgpuDevice, &descriptor) else {
			print("failed to create wgpu buffer")
			return .failure(.FailedToCreateBuffer)
		}

		// if we have bytes to copy get the mapping buffer and copy bytes to it
		if contentCount > 0 {
			if let contentData = contentData {
				guard let mappedPtr = wgpuBufferGetMappedRange(result, 0, contentCount) else {
					print("failed to get mapped range for wgpu buffer")
					return .failure(.FailedToMapBufferMemory)
				}
				defer { wgpuBufferUnmap(result) }

				if let error = contentData.withUnsafeBytes({ (contentPtr: UnsafeRawBufferPointer) in
					guard let baseAddress = contentPtr.baseAddress else {
						print("failed to get base address of content data")
						return CreateWGPUBufferInitError.FailedToGetContentBaseAddress
					}
					mappedPtr.copyMemory(from: baseAddress, byteCount: contentCount)
					return nil
				}) {
					return .failure(error)
				}
			}
		}

		return .success(result)
	}
}

func render(
	wgpuDevice: WGPUDevice, wgpuQueue: WGPUQueue, wgpuSurface: inout WGPUSurface,
	wgpuSurfaceConfiguration: inout WGPUSurfaceConfiguration, sdlWindow: OpaquePointer,
	wgpuRenderPipeline: WGPURenderPipeline,
	wgpuVertexBuffer: WGPUBuffer
) throws {
	var surfaceTexture = WGPUSurfaceTexture()
	withUnsafeMutablePointer(to: &surfaceTexture) { surfaceTexturePtr in
		wgpuSurfaceGetCurrentTexture(wgpuSurface, surfaceTexturePtr)
	}
	switch surfaceTexture.status {
	case WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal,
		WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal:
		break
	case WGPUSurfaceGetCurrentTextureStatus_Timeout, WGPUSurfaceGetCurrentTextureStatus_Outdated,
		WGPUSurfaceGetCurrentTextureStatus_Lost:
		print("surface texture status = \(surfaceTexture.status), resizing")
		resizeWGPUSurfaceConfiguration(
			wgpuSurface: &wgpuSurface, wgpuSurfaceConfiguration: &wgpuSurfaceConfiguration,
			sdlWindow: sdlWindow)
		return
	default:
		print("surface texture status = \(surfaceTexture.status), invalid")
		return
	}
	defer { wgpuTextureRelease(surfaceTexture.texture) }

	let textureView = try createWGPUTextureView(wgpuTexture: surfaceTexture.texture).get()
	defer { wgpuTextureViewRelease(textureView) }

	let commandEncoder = try createWGPUCommandEncoder(
		wgpuDevice: wgpuDevice, label: "command_encoder"
	).get()
	defer { wgpuCommandEncoderRelease(commandEncoder) }

	let renderPassEncoder = try createWGPURenderPassEncoder(
		wgpuCommandEncoder: commandEncoder,
		wgpuRenderColorPassAttachements: [
			{
				var result = WGPURenderPassColorAttachment()
				result.view = textureView
				result.loadOp = WGPULoadOp_Clear
				result.storeOp = WGPUStoreOp_Store
				result.depthSlice = WGPU_DEPTH_SLICE_UNDEFINED
				result.clearValue = WGPUColor(r: 0.25, g: 0.5, b: 1.0, a: 1.0)
				return result
			}()
		]
	).get()
	defer { wgpuRenderPassEncoderRelease(renderPassEncoder) }

	wgpuRenderPassEncoderSetPipeline(renderPassEncoder, wgpuRenderPipeline)
	wgpuRenderPassEncoderSetVertexBuffer(
		renderPassEncoder, 0, wgpuVertexBuffer, 0,
		// TODO don't hard-code byte size of buffer
		(2 + 4) * 3 * 4)
	wgpuRenderPassEncoderDraw(
		renderPassEncoder,
		// TODO don't hard-code number of vertices
		3, 1, 0, 0)
	wgpuRenderPassEncoderEnd(renderPassEncoder)

	let commandBuffer = try doWGPUCommandEncoderFinish(wgpuCommandEncoder: commandEncoder).get()
	defer { wgpuCommandBufferRelease(commandBuffer) }

	doWGPUQueueSubmit(wgpuQueue: wgpuQueue, wgpuCommandBuffers: [commandBuffer])

	let status = wgpuSurfacePresent(wgpuSurface)
	if status != WGPUStatus_Success {
		print("failed to present surface: \(status)")
	}
}

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
defer { wgpuInstanceRelease(wgpuInstance) }

var wgpuSurface = try createWGPUSurface(sdlWindow: sdlWindow!, wgpuInstance: wgpuInstance).get()
defer { wgpuSurfaceRelease(wgpuSurface) }

let wgpuAdapter = try createWGPUAdapter(wgpuInstance: wgpuInstance, wgpuSurface: wgpuSurface).get()
defer { wgpuAdapterRelease(wgpuAdapter) }

let wgpuDevice = try createWGPUDevice(wgpuAdapter: wgpuAdapter).get()
defer { wgpuDeviceRelease(wgpuDevice) }

let wgpuQueue = try createWGPUQueue(wgpuDevice: wgpuDevice).get()
defer { wgpuQueueRelease(wgpuQueue) }

let wgpuShader = try createWGPUShaderModuleWGSL(
	wgpuDevice: wgpuDevice,
	shaderSource: String(decoding: Data(PackageResources.shader_wsgl), as: UTF8.self)
).get()
defer { wgpuShaderModuleRelease(wgpuShader) }

let wgpuSurfaceCapabilities = try getWGPUSurfaceCapabilities(
	wgpuSurface: wgpuSurface, wgpuAdapter: wgpuAdapter
).get()
defer { wgpuSurfaceCapabilitiesFreeMembers(wgpuSurfaceCapabilities) }
let surfaceTextureFormat = wgpuSurfaceCapabilities.formats[0]

let wgpuPipelineLayout = try createWGPUPipelineLayout(wgpuDevice: wgpuDevice).get()
defer { wgpuPipelineLayoutRelease(wgpuPipelineLayout) }

let wgpuRenderPipeline = try createWGPURenderPipeline(
	wgpuDevice: wgpuDevice, wgpuPipelineLayout: wgpuPipelineLayout,
	wgpuShaderModule: wgpuShader, colorTargetFormat: surfaceTextureFormat
).get()
defer { wgpuRenderPipelineRelease(wgpuRenderPipeline) }

var wgpuSurfaceConfig = createWGPUSurfaceConfiguration(
	wgpuDevice: wgpuDevice, textureFormat: surfaceTextureFormat,
	alphaMode: wgpuSurfaceCapabilities.alphaModes[0]
)
defer { wgpuSurfaceRelease(wgpuSurface) }
resizeWGPUSurfaceConfiguration(
	wgpuSurface: &wgpuSurface, wgpuSurfaceConfiguration: &wgpuSurfaceConfig, sdlWindow: sdlWindow!)

var vertices = [
	Vertex(position: Vector2(x: -0.5, y: -0.5), color: RGBA(r: 1.0, g: 0.0, b: 0.0, a: 1.0)),
	Vertex(position: Vector2(x: 0.5, y: -0.5), color: RGBA(r: 0.0, g: 1.0, b: 0.0, a: 1.0)),
	Vertex(position: Vector2(x: 0, y: 0.5), color: RGBA(r: 0.0, g: 0.0, b: 1.0, a: 1.0)),
]
let vertexBuffer = try createWGPUBufferInit(
	wgpuDevice: wgpuDevice, content: &vertices,
	usage: WGPUBufferUsage_Vertex | WGPUBufferUsage_CopyDst
).get()
defer { wgpuBufferRelease(vertexBuffer) }

let framesPerSecond = 60
let delayBetweenFrames = UInt64(1000 / framesPerSecond)

var exiting = false
while !exiting {
	let startTicks = SDL_GetTicks()

	var e = SDL_Event.init()
	while SDL_PollEvent(&e) {
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

	try render(
		wgpuDevice: wgpuDevice, wgpuQueue: wgpuQueue, wgpuSurface: &wgpuSurface,
		wgpuSurfaceConfiguration: &wgpuSurfaceConfig, sdlWindow: sdlWindow!,
		wgpuRenderPipeline: wgpuRenderPipeline,
		wgpuVertexBuffer: vertexBuffer)

	let endTicks = SDL_GetTicks()
	let elapsedTicks = endTicks - startTicks
	if elapsedTicks < delayBetweenFrames {
		SDL_Delay(UInt32(delayBetweenFrames - elapsedTicks))
	}
}
