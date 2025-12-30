using System;
using System.IO;
using System.Runtime.InteropServices;
using Silk.NET.WebGPU;

namespace Experiment;

// TODO do logging correctly

public sealed unsafe class Pipeline : IDisposable
{
    private readonly App.State state;
    private RenderPipeline* renderPipeline;
    private ShaderModule* shaderModule;

    public Pipeline(App.State state)
    {
        this.state = state;
        shaderModule = CreateShaderModule();
        renderPipeline = BuildRenderPipeline();
    }

    public void Dispose()
    {
        if (renderPipeline != null)
        {
            state.WebGPU.RenderPipelineRelease(renderPipeline);
            renderPipeline = null;
        }

        if (shaderModule != null)
        {
            state.WebGPU.ShaderModuleRelease(shaderModule);
            shaderModule = null;
        }
    }

    public void Render(RenderPassEncoder* renderPassEncoder)
    {
        state.WebGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
        state.WebGPU.RenderPassEncoderDraw(renderPassEncoder, 3, 1, 0, 0);
    }

    private ShaderModule* CreateShaderModule()
    {
        var shaderCode = File.ReadAllText("Shaders/shader.wgsl");

        var shaderCodePtr = Marshal.StringToHGlobalAnsi(shaderCode);
        try
        {
            var wgslDescriptor = new ShaderModuleWGSLDescriptor
            {
                Code = (byte*)shaderCodePtr,
                Chain = { SType = SType.ShaderModuleWgslDescriptor },
            };

            var descriptor = new ShaderModuleDescriptor
            {
                NextInChain = (ChainedStruct*)&wgslDescriptor,
            };

            var module = state.WebGPU.DeviceCreateShaderModule(state.Device, &descriptor);

            Console.WriteLine("Shader module created");

            return module;
        }
        finally
        {
            if (shaderCodePtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(shaderCodePtr);
            }
        }
    }

    private RenderPipeline* BuildRenderPipeline()
    {
        var vertexEntryPointPtr = Marshal.StringToHGlobalAnsi("main_vs");
        var fragmentEntryPointPtr = Marshal.StringToHGlobalAnsi("main_fs");
        try
        {
            var vertexState = new VertexState
            {
                Module = shaderModule,
                EntryPoint = (byte*)vertexEntryPointPtr,
            };

            var blendState =
                stackalloc BlendState[1] {
                    new()
                    {
                        Color = new()
                        {
                            SrcFactor = BlendFactor.One,
                            DstFactor = BlendFactor.OneMinusSrcAlpha,
                            Operation = BlendOperation.Add,
                        },
                        Alpha = new()
                        {
                            SrcFactor = BlendFactor.One,
                            DstFactor = BlendFactor.OneMinusSrcAlpha,
                            Operation = BlendOperation.Add,
                        },
                    },
                };

            var colorTargetState =
                stackalloc ColorTargetState[1] {
                    new()
                    {
                        WriteMask = ColorWriteMask.All,
                        Format = state.PreferredTextureFormat,
                        Blend = blendState,
                    },
                };

            var fragmentState = new FragmentState
            {
                Module = shaderModule,
                EntryPoint = (byte*)fragmentEntryPointPtr,
                Targets = colorTargetState,
                TargetCount = 1,
            };

            var descriptor = new RenderPipelineDescriptor
            {
                Vertex = vertexState,
                Fragment = &fragmentState,
                Multisample = new MultisampleState
                {
                    Mask = 0x0FFFFFFF,
                    Count = 1,
                    AlphaToCoverageEnabled = false,
                },
                Primitive = new PrimitiveState
                {
                    CullMode = CullMode.Back,
                    FrontFace = FrontFace.Ccw,
                    Topology = PrimitiveTopology.TriangleList,
                },
            };

            renderPipeline = state.WebGPU.DeviceCreateRenderPipeline(state.Device, &descriptor);

            Console.WriteLine("Render pipeline created");

            return renderPipeline;
        }
        finally
        {
            if (vertexEntryPointPtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(vertexEntryPointPtr);
            }
            if (fragmentEntryPointPtr != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(fragmentEntryPointPtr);
            }
        }
    }
}
