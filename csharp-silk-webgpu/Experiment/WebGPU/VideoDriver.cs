namespace Experiment.WebGPU;

using System.Runtime.InteropServices;
using Silk.NET.WebGPU;
using Silk.NET.Windowing;

public unsafe class RenderPass
{
	public required RenderPassEncoder* RenderPassEncoder { get; init; }
}

public unsafe class VideoDriver : IVideoDriver
{
	private readonly WebGPU webGPU;
	private readonly Instance* instance;
	private readonly Surface* surface;
	private readonly Adapter* adapter;
	private readonly Device* device;
	private readonly TextureFormat surfaceTextureFormat;

	public VideoDriver(IWindow window)
	{
		webGPU = CreateWebGPU();
		instance = CreateInstance(webGPU);
		surface = CreateSurface(window, webGPU, instance);
		adapter = CreateAdapter(webGPU, instance, surface);
		device = CreateDevice(webGPU, adapter);
		surfaceTextureFormat = ConfigureSurface(window, webGPU, surface, device);
		ConfigureDebugCallback(webGPU, device);
	}

	public WebGPU WebGPU => webGPU;

	public Surface* Surface => surface;

	public Device* Device => device;

	public TextureFormat SurfaceTextureFormat => surfaceTextureFormat;

	public Queue* Queue => webGPU.DeviceGetQueue(device);

	public void Dispose()
	{
		webGPU.DeviceDestroy(device);
		webGPU.SurfaceRelease(surface);
		webGPU.AdapterRelease(adapter);
		webGPU.InstanceRelease(instance);
		Console.WriteLine("webGPU resources released");
	}

	public void RenderPass(Action<RenderPass> callback)
	{
		var commandEncoder = webGPU.DeviceCreateCommandEncoder(device, null);

		SurfaceTexture surfaceTexture = default;
		webGPU.SurfaceGetCurrentTexture(surface, ref surfaceTexture);

		var surfaceTextureView = webGPU.TextureCreateView(surfaceTexture.Texture, null);

		var colorAttachments = stackalloc RenderPassColorAttachment[] {
			new() {
				View = surfaceTextureView,
				LoadOp = LoadOp.Clear,
				ClearValue = System.Drawing.Color.CornflowerBlue.ToWebGPU(),
				StoreOp = StoreOp.Store,
			}
		};
		var renderPassDescriptor = new RenderPassDescriptor()
		{
			ColorAttachmentCount = 1,
			ColorAttachments = colorAttachments,
		};
		var renderPassEncoder = webGPU.CommandEncoderBeginRenderPass(commandEncoder, ref renderPassDescriptor);

		callback(new()
		{
			RenderPassEncoder = renderPassEncoder,
		});

		webGPU.RenderPassEncoderEnd(renderPassEncoder);

		var commandBuffer = webGPU.CommandEncoderFinish(commandEncoder, null);
		webGPU.QueueSubmit(Queue, 1, &commandBuffer);

		webGPU.SurfacePresent(surface);

		webGPU.TextureViewRelease(surfaceTextureView);
		webGPU.TextureRelease(surfaceTexture.Texture);
		webGPU.RenderPassEncoderRelease(renderPassEncoder);
		webGPU.CommandBufferRelease(commandBuffer);
		webGPU.CommandEncoderRelease(commandEncoder);
	}

	private static WebGPU CreateWebGPU()
	{
		return WebGPU.GetApi();
	}

	private static Instance* CreateInstance(WebGPU webGPU)
	{
		var descriptor = new InstanceDescriptor();
		var result = webGPU.CreateInstance(ref descriptor);
		Console.WriteLine("created instance");
		return result;
	}

	private static Surface* CreateSurface(IWindow window, WebGPU webGPU, Instance* instance)
	{
		var result = window.CreateWebGPUSurface(webGPU, instance);
		Console.WriteLine("created surface");
		return result;
	}

	private static Adapter* CreateAdapter(WebGPU webGPU, Instance* instance, Surface* surface)
	{
		Adapter* result = null;
		Exception? error = null;

		var options = new RequestAdapterOptions
		{
			CompatibleSurface = surface,
			BackendType = BackendType.Vulkan,
			PowerPreference = PowerPreference.HighPerformance
		};
		var callback = PfnRequestAdapterCallback.From((status, adapter, msgPtr, userDataPtr) =>
		{
			if (status == RequestAdapterStatus.Success)
			{
				result = adapter;
			}
			else
			{
				error = new Exception($"error getting adapter: {Marshal.PtrToStringAnsi((IntPtr)msgPtr)}");
			}
		});
		webGPU.InstanceRequestAdapter(instance, ref options, callback, null);

		if (error != null)
		{
			throw error;
		}
		if (result == null)
		{
			throw new Exception($"didn't create adapter, completed without callback being invoked");
		}

		var adapterProperties = new AdapterProperties();
		webGPU.AdapterGetProperties(result, ref adapterProperties);
		Console.WriteLine($"adapter type: {adapterProperties.AdapterType}");
		Console.WriteLine($"adapter architecture: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.Architecture)}");
		Console.WriteLine($"adapter backend type: {adapterProperties.BackendType}");
		Console.WriteLine($"adapter device ID: {adapterProperties.DeviceID}");
		Console.WriteLine($"adapter driver description: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.DriverDescription)}");
		Console.WriteLine($"adapter name: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.Name)}");
		Console.WriteLine($"adapter vendor ID: {adapterProperties.VendorID}");
		Console.WriteLine($"adapter vendor name: {Marshal.PtrToStringAnsi((IntPtr)adapterProperties.VendorName)}");

		return result;
	}

	private static Device* CreateDevice(WebGPU webGPU, Adapter* adapter)
	{
		Device* result = null;
		Exception? error = null;

		var descriptor = new DeviceDescriptor();
		var callback = PfnRequestDeviceCallback.From((status, device, msgPtr, userDataPtr) =>
		{
			if (status == RequestDeviceStatus.Success)
			{
				result = device;
			}
			else
			{
				error = new Exception($"error getting device: {Marshal.PtrToStringAnsi((IntPtr)msgPtr)}");
			}
		});
		webGPU.AdapterRequestDevice(adapter, ref descriptor, callback, null);

		if (error != null)
		{
			throw error;
		}
		if (result == null)
		{
			throw new Exception($"didn't create adapter, completed without callback being invoked");
		}

		Console.WriteLine("created device");

		return result;
	}

	private static TextureFormat ConfigureSurface(IWindow window, WebGPU webGPU, Surface* surface, Device* device)
	{
		var surfaceTextureFormat = TextureFormat.Bgra8Unorm;
		var configuration = new SurfaceConfiguration()
		{
			Device = device,
			Width = (uint)window.Size.X,
			Height = (uint)window.Size.Y,
			Format = surfaceTextureFormat,
			PresentMode = PresentMode.Fifo,
			Usage = TextureUsage.RenderAttachment,
		};
		webGPU.SurfaceConfigure(surface, ref configuration);
		return surfaceTextureFormat;
	}

	private static void ConfigureDebugCallback(WebGPU webGPU, Device* device)
	{
		var callback = PfnErrorCallback.From((type, msgPtr, userDataPtr) =>
		{
			Console.WriteLine($"unhandled WebGPU error: {Marshal.PtrToStringAnsi((IntPtr)msgPtr)}");
		});
		webGPU.DeviceSetUncapturedErrorCallback(device, callback, null);
	}
}