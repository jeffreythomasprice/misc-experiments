import CWGPU

@MainActor
func createWGPUInstance() -> WGPUInstance {
	let result = wgpuCreateInstance(nil)!
	print("created wgpu instance")
	return result
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
	wgpuDevice: WGPUDevice, label: String? = nil, content: [T], usage: WGPUBufferUsage
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
