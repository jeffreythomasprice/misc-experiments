import CSDL
import CWGPU
import Foundation

struct Vertex {
	var position: Vector2<Float>
	var color: RGBA<Float>
}

class Buffer<T> {
	let buffer: WGPUBuffer
	let count: Int
	let byteCount: Int

	init(wgpuDevice: WGPUDevice, label: String? = nil, content: [T], usage: WGPUBufferUsage)
		throws
	{
		buffer = try createWGPUBufferInit(
			wgpuDevice: wgpuDevice, label: label, content: content, usage: usage
		).get()
		count = content.count
		byteCount = count * MemoryLayout<T>.stride
	}

	deinit {
		wgpuBufferRelease(buffer)
	}
}

func getCurrentOrthoMatrix(sdlWindow: OpaquePointer) -> Matrix4<Float32> {
	var width: Int32 = 0
	var height: Int32 = 0
	SDL_GetWindowSize(sdlWindow, &width, &height)
	return Matrix4<Float32>.ortho(
		left: 0, right: Float32(width), bottom: Float32(height), top: 0, near: -1, far: 1)
}

func render(
	wgpuDevice: WGPUDevice, wgpuQueue: WGPUQueue, wgpuSurface: inout WGPUSurface,
	wgpuSurfaceConfiguration: inout WGPUSurfaceConfiguration, sdlWindow: OpaquePointer,
	wgpuRenderPipeline: WGPURenderPipeline,
	vertexBuffer: Buffer<Vertex>,
	indexBuffer: Buffer<UInt16>,
	sceneUniformBuffer: Buffer<Matrix4<Float32>>,
	sceneUniformBindGroup: WGPUBindGroup,
	modelUniformBuffer: Buffer<Matrix4<Float32>>,
	modelUniformBindGroup: WGPUBindGroup
) throws {
	var projectionMatrix = getCurrentOrthoMatrix(sdlWindow: sdlWindow)
	withUnsafePointer(to: &projectionMatrix) {
		wgpuQueueWriteBuffer(
			wgpuQueue, sceneUniformBuffer.buffer, 0, $0,
			MemoryLayout<Matrix4<Float32>>.size)
	}

	var orthoMatrix = Matrix4<Float32>.identity
	withUnsafePointer(to: &orthoMatrix) {
		wgpuQueueWriteBuffer(
			wgpuQueue, modelUniformBuffer.buffer, 0, $0,
			MemoryLayout<Matrix4<Float32>>.size)
	}

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
	wgpuRenderPassEncoderSetBindGroup(renderPassEncoder, 0, sceneUniformBindGroup, 0, nil)
	wgpuRenderPassEncoderSetBindGroup(renderPassEncoder, 1, modelUniformBindGroup, 0, nil)
	wgpuRenderPassEncoderSetVertexBuffer(
		renderPassEncoder, 0, vertexBuffer.buffer, 0,
		UInt64(vertexBuffer.byteCount))
	wgpuRenderPassEncoderSetIndexBuffer(
		renderPassEncoder, indexBuffer.buffer, WGPUIndexFormat_Uint16, 0,
		UInt64(indexBuffer.byteCount))
	wgpuRenderPassEncoderDrawIndexed(renderPassEncoder, UInt32(indexBuffer.count), 1, 0, 0, 0)
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

// TODO helper that builds the buffer, layout, and bind group all at the same time and releases them all together
let sceneUniformBindingGroupLayout = createWGPUBindGroupLayout(
	wgpuDevice: wgpuDevice, label: "scene_uniform_bind_group_layout",
	entries: [
		{
			var result = WGPUBindGroupLayoutEntry()
			result.binding = 0
			result.visibility = WGPUShaderStage_Vertex
			result.buffer.type = WGPUBufferBindingType_Uniform
			result.buffer.hasDynamicOffset = 0
			// TODO could be 0? determine from buffer generic type?
			result.buffer.minBindingSize = UInt64(MemoryLayout<Matrix4<Float32>>.size)
			return result
		}()
	])
let sceneUniformBuffer = try Buffer<Matrix4<Float32>>(
	wgpuDevice: wgpuDevice, label: "scene_uniform_buffer",
	content: [
		Matrix4<Float32>.identity
	],
	usage: WGPUBufferUsage_Uniform | WGPUBufferUsage_CopyDst
)
let sceneUniformBindGroup = createWGPUBindGroup(
	wgpuDevice: wgpuDevice, label: "scene_uniform_bind_group",
	layout: sceneUniformBindingGroupLayout,
	entries: [
		{
			var result = WGPUBindGroupEntry()
			result.buffer = sceneUniformBuffer.buffer
			result.offset = 0
			result.size = UInt64(sceneUniformBuffer.byteCount)
			// TODO use the same constant as the layout object
			result.binding = 0
			return result
		}()
	])
defer {
	wgpuBindGroupLayoutRelease(sceneUniformBindingGroupLayout)
	wgpuBindGroupRelease(sceneUniformBindGroup)
}
let modelUniformBindingGroupLayout = createWGPUBindGroupLayout(
	wgpuDevice: wgpuDevice, label: "model_uniform_bind_group_layout",
	entries: [
		{
			print("TODO min binding size of mat4 = \(MemoryLayout<Matrix4<Float32>>.size)")
			var result = WGPUBindGroupLayoutEntry()
			result.binding = 0
			result.visibility = WGPUShaderStage_Vertex
			result.buffer.type = WGPUBufferBindingType_Uniform
			result.buffer.hasDynamicOffset = 0
			// TODO could be 0? determine from buffer generic type?
			result.buffer.minBindingSize = UInt64(MemoryLayout<Matrix4<Float32>>.size)
			return result
		}()
	])
let modelUniformBuffer = try Buffer<Matrix4<Float32>>(
	wgpuDevice: wgpuDevice, label: "model_uniform_buffer",
	content: [
		Matrix4<Float32>.identity
	],
	usage: WGPUBufferUsage_Uniform | WGPUBufferUsage_CopyDst
)
let modelUniformBindGroup = createWGPUBindGroup(
	wgpuDevice: wgpuDevice, label: "model_uniform_bind_group",
	layout: modelUniformBindingGroupLayout,
	entries: [
		{
			var result = WGPUBindGroupEntry()
			result.buffer = modelUniformBuffer.buffer
			result.offset = 0
			result.size = UInt64(modelUniformBuffer.byteCount)
			// TODO use the same constant as the layout object
			result.binding = 0
			return result
		}()
	])
defer {
	wgpuBindGroupLayoutRelease(modelUniformBindingGroupLayout)
	wgpuBindGroupRelease(modelUniformBindGroup)
}
let wgpuPipelineLayout = try createWGPUPipelineLayout(
	wgpuDevice: wgpuDevice,
	bindGroupLayouts: [
		sceneUniformBindingGroupLayout,
		modelUniformBindingGroupLayout,
	]
)
.get()
defer { wgpuPipelineLayoutRelease(wgpuPipelineLayout) }

let wgpuRenderPipeline = try createWGPURenderPipeline(
	wgpuDevice: wgpuDevice, wgpuPipelineLayout: wgpuPipelineLayout,
	wgpuShaderModule: wgpuShader,
	vertices: VertexPipelineInit(
		entryPoint: "vs_main",
		buffers: [
			VertexBufferInit(
				stepMode: WGPUVertexStepMode_Vertex,
				stride: UInt64(MemoryLayout<Vertex>.stride),
				attributes: [
					WGPUVertexAttribute(
						format: WGPUVertexFormat_Float32x2,
						offset: UInt64(MemoryLayout<Vertex>.offset(of: \Vertex.position)!),
						shaderLocation: 0),
					WGPUVertexAttribute(
						format: WGPUVertexFormat_Float32x4,
						offset: UInt64(MemoryLayout<Vertex>.offset(of: \Vertex.color)!),
						shaderLocation: 1),
				]
			)
		]),
	fragments: FragmentPipelineInit(
		entryPoint: "fs_main",
		targets: [
			FragmentColorTargetStateInit(
				format: surfaceTextureFormat, writeMask: WGPUColorWriteMask_All)
		])
).get()
defer { wgpuRenderPipelineRelease(wgpuRenderPipeline) }

var wgpuSurfaceConfig = createWGPUSurfaceConfiguration(
	wgpuDevice: wgpuDevice, textureFormat: surfaceTextureFormat,
	alphaMode: wgpuSurfaceCapabilities.alphaModes[0]
)
defer { wgpuSurfaceRelease(wgpuSurface) }
resizeWGPUSurfaceConfiguration(
	wgpuSurface: &wgpuSurface, wgpuSurfaceConfiguration: &wgpuSurfaceConfig, sdlWindow: sdlWindow!)

let vertexBuffer = try Buffer<Vertex>(
	wgpuDevice: wgpuDevice, label: "vertex_buffer",
	content: [
		Vertex(position: Vector2(x: 50, y: 50), color: RGBA(r: 1.0, g: 0.0, b: 0.0, a: 1.0)),
		Vertex(position: Vector2(x: 50, y: 300), color: RGBA(r: 0.0, g: 1.0, b: 0.0, a: 1.0)),
		Vertex(position: Vector2(x: 300, y: 300), color: RGBA(r: 0.0, g: 0.0, b: 1.0, a: 1.0)),
		Vertex(position: Vector2(x: 300, y: 50), color: RGBA(r: 1.0, g: 0.0, b: 1.0, a: 1.0)),
	],
	usage: WGPUBufferUsage_Vertex | WGPUBufferUsage_CopyDst
)

let indexBuffer = try Buffer<UInt16>(
	wgpuDevice: wgpuDevice, label: "index_buffer",
	content: [0, 1, 2, 2, 3, 0],
	usage: WGPUBufferUsage_Index | WGPUBufferUsage_CopyDst
)

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
		vertexBuffer: vertexBuffer, indexBuffer: indexBuffer,
		sceneUniformBuffer: sceneUniformBuffer, sceneUniformBindGroup: sceneUniformBindGroup,
		modelUniformBuffer: modelUniformBuffer, modelUniformBindGroup: modelUniformBindGroup
	)

	let endTicks = SDL_GetTicks()
	let elapsedTicks = endTicks - startTicks
	if elapsedTicks < delayBetweenFrames {
		SDL_Delay(UInt32(delayBetweenFrames - elapsedTicks))
	}
}
