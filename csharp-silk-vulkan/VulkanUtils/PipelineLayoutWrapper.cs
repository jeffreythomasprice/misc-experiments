namespace Experiment.VulkanUtils;

using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using Experiment.VulkanUtils;
using Silk.NET.Core;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Native;
using Silk.NET.Maths;
using Silk.NET.Vulkan;
using Silk.NET.Vulkan.Extensions.EXT;
using Silk.NET.Vulkan.Extensions.KHR;
using Silk.NET.Windowing;

public sealed unsafe class PipelineLayoutWrapper : IDisposable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    private readonly PipelineLayout pipelineLayout;

    public PipelineLayoutWrapper(
        Vk vk,
        DeviceWrapper device,
        byte[] vertexShaderSpirvBytes,
        byte[] fragmentShaderSpirvBytes
    )
    {
        this.vk = vk;
        this.device = device;

        using var vertexShaderModule = new ShaderModuleWrapper(vk, device, vertexShaderSpirvBytes);
        using var fragmentShaderModule = new ShaderModuleWrapper(
            vk,
            device,
            fragmentShaderSpirvBytes
        );

        using var vertexShaderMainName = new PointerUtils.DisposableStringPointer("main");
        var vertShaderStageInfo = new PipelineShaderStageCreateInfo()
        {
            SType = StructureType.PipelineShaderStageCreateInfo,
            Stage = ShaderStageFlags.VertexBit,
            Module = vertexShaderModule.ShaderModule,
            PName = (byte*)vertexShaderMainName.Pointer,
        };

        using var fragmentShaderMainName = new PointerUtils.DisposableStringPointer("main");
        var fragShaderStageInfo = new PipelineShaderStageCreateInfo()
        {
            SType = StructureType.PipelineShaderStageCreateInfo,
            Stage = ShaderStageFlags.FragmentBit,
            Module = fragmentShaderModule.ShaderModule,
            PName = (byte*)fragmentShaderMainName.Pointer,
        };

        // TODO but where do shaderStages get used?
        var shaderStages = stackalloc[] { vertShaderStageInfo, fragShaderStageInfo };

        // TODO impl

        //   PipelineVertexInputStateCreateInfo vertexInputInfo = new()
        // {
        //     SType = StructureType.PipelineVertexInputStateCreateInfo,
        //     VertexBindingDescriptionCount = 0,
        //     VertexAttributeDescriptionCount = 0,
        // };

        // PipelineInputAssemblyStateCreateInfo inputAssembly = new()
        // {
        //     SType = StructureType.PipelineInputAssemblyStateCreateInfo,
        //     Topology = PrimitiveTopology.TriangleList,
        //     PrimitiveRestartEnable = false,
        // };

        // Viewport viewport = new()
        // {
        //     X = 0,
        //     Y = 0,
        //     Width = swapChainExtent.Width,
        //     Height = swapChainExtent.Height,
        //     MinDepth = 0,
        //     MaxDepth = 1,
        // };

        // Rect2D scissor = new()
        // {
        //     Offset = { X = 0, Y = 0 },
        //     Extent = swapChainExtent,
        // };

        // PipelineViewportStateCreateInfo viewportState = new()
        // {
        //     SType = StructureType.PipelineViewportStateCreateInfo,
        //     ViewportCount = 1,
        //     PViewports = &viewport,
        //     ScissorCount = 1,
        //     PScissors = &scissor,
        // };

        // PipelineRasterizationStateCreateInfo rasterizer = new()
        // {
        //     SType = StructureType.PipelineRasterizationStateCreateInfo,
        //     DepthClampEnable = false,
        //     RasterizerDiscardEnable = false,
        //     PolygonMode = PolygonMode.Fill,
        //     LineWidth = 1,
        //     CullMode = CullModeFlags.BackBit,
        //     FrontFace = FrontFace.Clockwise,
        //     DepthBiasEnable = false,
        // };

        // PipelineMultisampleStateCreateInfo multisampling = new()
        // {
        //     SType = StructureType.PipelineMultisampleStateCreateInfo,
        //     SampleShadingEnable = false,
        //     RasterizationSamples = SampleCountFlags.Count1Bit,
        // };

        // PipelineColorBlendAttachmentState colorBlendAttachment = new()
        // {
        //     ColorWriteMask = ColorComponentFlags.RBit | ColorComponentFlags.GBit | ColorComponentFlags.BBit | ColorComponentFlags.ABit,
        //     BlendEnable = false,
        // };

        // PipelineColorBlendStateCreateInfo colorBlending = new()
        // {
        //     SType = StructureType.PipelineColorBlendStateCreateInfo,
        //     LogicOpEnable = false,
        //     LogicOp = LogicOp.Copy,
        //     AttachmentCount = 1,
        //     PAttachments = &colorBlendAttachment,
        // };

        // colorBlending.BlendConstants[0] = 0;
        // colorBlending.BlendConstants[1] = 0;
        // colorBlending.BlendConstants[2] = 0;
        // colorBlending.BlendConstants[3] = 0;

        // PipelineLayoutCreateInfo pipelineLayoutInfo = new()
        // {
        //     SType = StructureType.PipelineLayoutCreateInfo,
        //     SetLayoutCount = 0,
        //     PushConstantRangeCount = 0,
        // };

        // if (vk.CreatePipelineLayout(device, in pipelineLayoutInfo, null, out pipelineLayout) != Result.Success)
        // {
        //     throw new Exception("failed to create pipeline layout!");
        // }

        throw new NotImplementedException();
    }

    public void Dispose()
    {
        vk.DestroyPipelineLayout(device.Device, pipelineLayout, null);
    }
}
