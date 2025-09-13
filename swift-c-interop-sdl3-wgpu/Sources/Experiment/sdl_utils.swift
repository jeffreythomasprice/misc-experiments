import CSDL
import CWGPU

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
