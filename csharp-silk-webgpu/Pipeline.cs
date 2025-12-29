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
            state.WGPU.RenderPipelineRelease(renderPipeline);
            renderPipeline = null;
        }

        if (shaderModule != null)
        {
            state.WGPU.ShaderModuleRelease(shaderModule);
            shaderModule = null;
        }
    }

    public void Render(RenderPassEncoder* renderPassEncoder)
    {
        state.WGPU.RenderPassEncoderSetPipeline(renderPassEncoder, renderPipeline);
        state.WGPU.RenderPassEncoderDraw(renderPassEncoder, 3, 1, 0, 0);
    }

    private ShaderModule* CreateShaderModule()
    {
        var shaderCode = File.ReadAllText("Shaders/shader.wgsl");

        var shaderCodePtr = Marshal.StringToHGlobalAnsi(shaderCode);

        ShaderModuleWGSLDescriptor wgslDescriptor = new ShaderModuleWGSLDescriptor();
        wgslDescriptor.Code = (byte*)shaderCodePtr;
        wgslDescriptor.Chain.SType = SType.ShaderModuleWgslDescriptor;

        ShaderModuleDescriptor descriptor = new ShaderModuleDescriptor();
        descriptor.NextInChain = (ChainedStruct*)&wgslDescriptor;

        var module = state.WGPU.DeviceCreateShaderModule(state.Device, &descriptor);

        Marshal.FreeHGlobal(shaderCodePtr);

        Console.WriteLine("Shader module created");

        return module;
    }

    private RenderPipeline* BuildRenderPipeline()
    {
        var vertexEntryPointPtr = Marshal.StringToHGlobalAnsi("main_vs");
        var fragmentEntryPointPtr = Marshal.StringToHGlobalAnsi("main_fs");

        VertexState vertexState = new VertexState();
        vertexState.Module = shaderModule;
        vertexState.EntryPoint = (byte*)vertexEntryPointPtr;

        BlendState* blendState = stackalloc BlendState[1];
        blendState[0].Color = new BlendComponent
        {
            SrcFactor = BlendFactor.One,
            DstFactor = BlendFactor.OneMinusSrcAlpha,
            Operation = BlendOperation.Add,
        };
        blendState[0].Alpha = new BlendComponent
        {
            SrcFactor = BlendFactor.One,
            DstFactor = BlendFactor.OneMinusSrcAlpha,
            Operation = BlendOperation.Add,
        };

        ColorTargetState* colorTargetState = stackalloc ColorTargetState[1];
        colorTargetState[0].WriteMask = ColorWriteMask.All;
        colorTargetState[0].Format = state.PreferredTextureFormat;
        colorTargetState[0].Blend = blendState;

        FragmentState fragmentState = new FragmentState();
        fragmentState.Module = shaderModule;
        fragmentState.EntryPoint = (byte*)fragmentEntryPointPtr;
        fragmentState.Targets = colorTargetState;
        fragmentState.TargetCount = 1;

        RenderPipelineDescriptor descriptor = new RenderPipelineDescriptor();
        descriptor.Vertex = vertexState;
        descriptor.Fragment = &fragmentState;
        descriptor.Multisample = new MultisampleState
        {
            Mask = 0x0FFFFFFF,
            Count = 1,
            AlphaToCoverageEnabled = false,
        };
        descriptor.Primitive = new PrimitiveState
        {
            CullMode = CullMode.Back,
            FrontFace = FrontFace.Ccw,
            Topology = PrimitiveTopology.TriangleList,
        };

        renderPipeline = state.WGPU.DeviceCreateRenderPipeline(state.Device, &descriptor);

        Marshal.FreeHGlobal(vertexEntryPointPtr);
        Marshal.FreeHGlobal(fragmentEntryPointPtr);

        Console.WriteLine("Render pipeline created");

        return renderPipeline;
    }
}
