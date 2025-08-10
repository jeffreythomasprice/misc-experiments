package main

import (
	"fmt"
	"os"
	"runtime"
	"unsafe"

	"github.com/cogentcore/webgpu/wgpu"

	_ "embed"
)

var forceFallbackAdapter = os.Getenv("WGPU_FORCE_FALLBACK_ADAPTER") == "1"

func init() {
	runtime.LockOSThread()

	switch os.Getenv("WGPU_LOG_LEVEL") {
	case "OFF":
		wgpu.SetLogLevel(wgpu.LogLevelOff)
	case "ERROR":
		wgpu.SetLogLevel(wgpu.LogLevelError)
	case "WARN":
		wgpu.SetLogLevel(wgpu.LogLevelWarn)
	case "INFO":
		wgpu.SetLogLevel(wgpu.LogLevelInfo)
	case "DEBUG":
		wgpu.SetLogLevel(wgpu.LogLevelDebug)
	case "TRACE":
		wgpu.SetLogLevel(wgpu.LogLevelTrace)
	}
}

//go:embed shader.wgsl
var shader string

type Vector2f struct {
	X float32
	Y float32
}

type RGBAf struct {
	R float32
	G float32
	B float32
	A float32
}

type Vertex struct {
	Position Vector2f
	Color    RGBAf
}

type State struct {
	instance *wgpu.Instance
	adapter  *wgpu.Adapter
	surface  *wgpu.Surface
	device   *wgpu.Device
	queue    *wgpu.Queue
	config   *wgpu.SurfaceConfiguration
	buffer   *Buffer[Vertex]
	pipeline *wgpu.RenderPipeline
}

func InitState[T interface{ GetSize() (int, int) }](window T, sd *wgpu.SurfaceDescriptor) (result *State, err error) {
	defer func() {
		if err != nil {
			result.Destroy()
			result = nil
		}
	}()
	result = &State{}

	result.instance = wgpu.CreateInstance(nil)

	result.surface = result.instance.CreateSurface(sd)

	result.adapter, err = result.instance.RequestAdapter(&wgpu.RequestAdapterOptions{
		ForceFallbackAdapter: forceFallbackAdapter,
		CompatibleSurface:    result.surface,
	})
	if err != nil {
		return result, err
	}
	defer result.adapter.Release()

	adapterInfo := result.adapter.GetInfo()
	fmt.Printf("Adapter Type: %v\n", adapterInfo.AdapterType)
	fmt.Printf("Adapter Architecture: %v\n", adapterInfo.Architecture)
	fmt.Printf("Adapter Backend Type: %v\n", adapterInfo.BackendType)
	fmt.Printf("Adapter Device ID: %v\n", adapterInfo.DeviceId)
	fmt.Printf("Adapter Driver Description: %v\n", adapterInfo.DriverDescription)
	fmt.Printf("Adapter Name: %v\n", adapterInfo.Name)
	fmt.Printf("Adapter Vendor ID: %v\n", adapterInfo.VendorId)
	fmt.Printf("Adapter Vendor Name: %v\n", adapterInfo.VendorName)

	result.device, err = result.adapter.RequestDevice(nil)
	if err != nil {
		return result, err
	}
	result.queue = result.device.GetQueue()

	shader, err := result.device.CreateShaderModule(&wgpu.ShaderModuleDescriptor{
		Label:          "shader.wgsl",
		WGSLDescriptor: &wgpu.ShaderModuleWGSLDescriptor{Code: shader},
	})
	if err != nil {
		return result, err
	}
	defer shader.Release()

	caps := result.surface.GetCapabilities(result.adapter)

	width, height := window.GetSize()
	result.config = &wgpu.SurfaceConfiguration{
		Usage:       wgpu.TextureUsageRenderAttachment,
		Format:      caps.Formats[0],
		Width:       uint32(width),
		Height:      uint32(height),
		PresentMode: wgpu.PresentModeFifo,
		AlphaMode:   caps.AlphaModes[0],
	}

	result.surface.Configure(result.adapter, result.device, result.config)

	result.buffer, err = NewBufferInit(
		result.device,
		[]Vertex{
			{
				Position: Vector2f{
					X: -0.5,
					Y: -0.5,
				},
				Color: RGBAf{
					R: 1,
					G: 0,
					B: 0,
					A: 1,
				},
			}, {
				Position: Vector2f{
					X: 0.5,
					Y: -0.5,
				},
				Color: RGBAf{
					R: 0,
					G: 1,
					B: 0,
					A: 1,
				},
			}, {
				Position: Vector2f{
					X: 0.0,
					Y: 0.5,
				},
				Color: RGBAf{
					R: 0,
					G: 0,
					B: 1,
					A: 1,
				},
			},
		},
		wgpu.BufferUsageVertex,
	)
	if err != nil {
		return result, err
	}

	result.pipeline, err = result.device.CreateRenderPipeline(&wgpu.RenderPipelineDescriptor{
		Label: "Render Pipeline",
		Vertex: wgpu.VertexState{
			Module:     shader,
			EntryPoint: "vs_main",
			Buffers: []wgpu.VertexBufferLayout{
				{
					StepMode:    wgpu.VertexStepModeVertex,
					ArrayStride: uint64(result.buffer.StrideInBytes),
					Attributes: []wgpu.VertexAttribute{
						{
							Format:         wgpu.VertexFormatFloat32x2,
							Offset:         uint64(unsafe.Offsetof(Vertex{}.Position)),
							ShaderLocation: 0,
						}, {
							Format:         wgpu.VertexFormatFloat32x4,
							Offset:         uint64(unsafe.Offsetof(Vertex{}.Color)),
							ShaderLocation: 1,
						},
					},
				},
			},
		},
		Fragment: &wgpu.FragmentState{
			Module:     shader,
			EntryPoint: "fs_main",
			Targets: []wgpu.ColorTargetState{
				{
					Format:    result.config.Format,
					Blend:     &wgpu.BlendStateReplace,
					WriteMask: wgpu.ColorWriteMaskAll,
				},
			},
		},
		Primitive: wgpu.PrimitiveState{
			Topology:         wgpu.PrimitiveTopologyTriangleList,
			StripIndexFormat: wgpu.IndexFormatUndefined,
			FrontFace:        wgpu.FrontFaceCCW,
			CullMode:         wgpu.CullModeNone,
		},
		Multisample: wgpu.MultisampleState{
			Count:                  1,
			Mask:                   0xFFFFFFFF,
			AlphaToCoverageEnabled: false,
		},
	})
	if err != nil {
		return result, err
	}

	return result, nil
}

func (s *State) Resize(width, height int) {
	if width > 0 && height > 0 {
		s.config.Width = uint32(width)
		s.config.Height = uint32(height)

		s.surface.Configure(s.adapter, s.device, s.config)
	}
}

func (s *State) Render() error {
	nextTexture, err := s.surface.GetCurrentTexture()
	if err != nil {
		return err
	}
	view, err := nextTexture.CreateView(nil)
	if err != nil {
		return err
	}
	defer view.Release()

	encoder, err := s.device.CreateCommandEncoder(&wgpu.CommandEncoderDescriptor{
		Label: "Command Encoder",
	})
	if err != nil {
		return err
	}
	defer encoder.Release()

	renderPass := encoder.BeginRenderPass(&wgpu.RenderPassDescriptor{
		ColorAttachments: []wgpu.RenderPassColorAttachment{
			{
				View:    view,
				LoadOp:  wgpu.LoadOpClear,
				StoreOp: wgpu.StoreOpStore,
				ClearValue: wgpu.Color{
					R: 0.25,
					G: 0.5,
					B: 1,
					A: 1,
				},
			},
		},
	})

	renderPass.SetPipeline(s.pipeline)
	renderPass.SetVertexBuffer(
		0,
		s.buffer.Buffer,
		0,
		uint64(s.buffer.StrideInBytes)*uint64(s.buffer.Length),
	)
	renderPass.Draw(
		uint32(s.buffer.Length),
		1,
		0,
		0,
	)
	renderPass.End()
	renderPass.Release() // must release

	cmdBuffer, err := encoder.Finish(nil)
	if err != nil {
		return err
	}
	defer cmdBuffer.Release()

	s.queue.Submit(cmdBuffer)
	s.surface.Present()

	return nil
}

func (s *State) Destroy() {
	if s.pipeline != nil {
		s.pipeline.Release()
		s.pipeline = nil
	}
	if s.config != nil {
		s.config = nil
	}
	if s.queue != nil {
		s.queue.Release()
		s.queue = nil
	}
	if s.device != nil {
		s.device.Release()
		s.device = nil
	}
	if s.surface != nil {
		s.surface.Release()
		s.surface = nil
	}
	if s.instance != nil {
		s.instance.Release()
		s.instance = nil
	}
	if s.buffer != nil {
		s.buffer.Destroy()
		s.buffer = nil
	}
}
