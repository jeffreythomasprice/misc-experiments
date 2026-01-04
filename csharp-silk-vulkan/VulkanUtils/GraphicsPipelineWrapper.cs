namespace Experiment.VulkanUtils;

using System;
using Silk.NET.Vulkan;

public sealed unsafe class GraphicsPipelineWrapper<TVertex> : IDisposable
    where TVertex : IBufferBindable
{
    private readonly Vk vk;
    private readonly DeviceWrapper device;
    public readonly PipelineLayout PipelineLayout;
    public readonly Pipeline GraphicsPipeline;

    public GraphicsPipelineWrapper(
        Vk vk,
        DeviceWrapper device,
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        byte[] vertexShaderSpirvBytes,
        byte[] fragmentShaderSpirvBytes,
        DescriptorSetLayoutWrapper[] descriptorSetLayouts
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

        var shaderStages = stackalloc[] { vertShaderStageInfo, fragShaderStageInfo };

        VertexInputBindingDescription bindingDescription = TVertex.BindingDescription;
        var attributeDescriptions = TVertex.AttributeDescriptions;
        fixed (VertexInputAttributeDescription* attributeDescriptionsPtr = attributeDescriptions)
        {
            var vertexInputInfo = new PipelineVertexInputStateCreateInfo()
            {
                SType = StructureType.PipelineVertexInputStateCreateInfo,
                VertexBindingDescriptionCount = 1,
                PVertexBindingDescriptions = &bindingDescription,
                VertexAttributeDescriptionCount = (uint)attributeDescriptions.Length,
                PVertexAttributeDescriptions = attributeDescriptionsPtr,
            };

            var inputAssembly = new PipelineInputAssemblyStateCreateInfo()
            {
                SType = StructureType.PipelineInputAssemblyStateCreateInfo,
                Topology = PrimitiveTopology.TriangleList,
                PrimitiveRestartEnable = false,
            };

            var viewport = new Viewport()
            {
                X = 0,
                Width = swapchain.Extent.Width,

                /*
                the default clip space in vulkan puts positive Y pointing towards the bottom of the screen
                in OpenGL this is backwards, the positive Y goes up towards the top of the screen
                we're going to flip this around so that normalized device coordinates coming out of fragment shaders have
                    (-1, -1) at the top left corner, and
                    (1,1) at the bottom right corner
                */
                Y = swapchain.Extent.Height,
                Height = -swapchain.Extent.Height,

                MinDepth = 0,
                MaxDepth = 1,
            };

            var scissor = new Rect2D() { Offset = { X = 0, Y = 0 }, Extent = swapchain.Extent };

            var viewportState = new PipelineViewportStateCreateInfo()
            {
                SType = StructureType.PipelineViewportStateCreateInfo,
                ViewportCount = 1,
                PViewports = &viewport,
                ScissorCount = 1,
                PScissors = &scissor,
            };

            var rasterizer = new PipelineRasterizationStateCreateInfo()
            {
                SType = StructureType.PipelineRasterizationStateCreateInfo,
                DepthClampEnable = false,
                RasterizerDiscardEnable = false,
                PolygonMode = PolygonMode.Fill,
                LineWidth = 1,
                CullMode = CullModeFlags.BackBit,
                FrontFace = FrontFace.Clockwise,
                DepthBiasEnable = false,
            };

            var multisampling = new PipelineMultisampleStateCreateInfo()
            {
                SType = StructureType.PipelineMultisampleStateCreateInfo,
                SampleShadingEnable = false,
                RasterizationSamples = SampleCountFlags.Count1Bit,
            };

            var colorBlendAttachment = new PipelineColorBlendAttachmentState()
            {
                ColorWriteMask =
                    ColorComponentFlags.RBit
                    | ColorComponentFlags.GBit
                    | ColorComponentFlags.BBit
                    | ColorComponentFlags.ABit,
                BlendEnable = false,
            };

            var colorBlending = new PipelineColorBlendStateCreateInfo()
            {
                SType = StructureType.PipelineColorBlendStateCreateInfo,
                LogicOpEnable = false,
                LogicOp = LogicOp.Copy,
                AttachmentCount = 1,
                PAttachments = &colorBlendAttachment,
            };
            colorBlending.BlendConstants[0] = 0;
            colorBlending.BlendConstants[1] = 0;
            colorBlending.BlendConstants[2] = 0;
            colorBlending.BlendConstants[3] = 0;

            fixed (
                DescriptorSetLayout* descriptorSetLayoutsPtr = descriptorSetLayouts
                    .Select(x => x.DescriptorSetLayout)
                    .ToArray()
            )
            {
                var pipelineLayoutInfo = new PipelineLayoutCreateInfo()
                {
                    SType = StructureType.PipelineLayoutCreateInfo,
                    SetLayoutCount = (uint)descriptorSetLayouts.Length,
                    PSetLayouts = descriptorSetLayoutsPtr,
                    PushConstantRangeCount = 0,
                };

                if (
                    vk.CreatePipelineLayout(
                        device.Device,
                        in pipelineLayoutInfo,
                        null,
                        out PipelineLayout
                    ) != Result.Success
                )
                {
                    throw new Exception("failed to create pipeline layout");
                }

                var pipelineInfo = new GraphicsPipelineCreateInfo()
                {
                    SType = StructureType.GraphicsPipelineCreateInfo,
                    StageCount = 2,
                    PStages = shaderStages,
                    PVertexInputState = &vertexInputInfo,
                    PInputAssemblyState = &inputAssembly,
                    PViewportState = &viewportState,
                    PRasterizationState = &rasterizer,
                    PMultisampleState = &multisampling,
                    PColorBlendState = &colorBlending,
                    Layout = PipelineLayout,
                    RenderPass = renderPass.RenderPass,
                    Subpass = 0,
                    BasePipelineHandle = default,
                };

                if (
                    vk.CreateGraphicsPipelines(
                        device.Device,
                        default,
                        1,
                        in pipelineInfo,
                        null,
                        out GraphicsPipeline
                    ) != Result.Success
                )
                {
                    throw new Exception("failed to create graphics pipeline");
                }
            }
        }
    }

    public void Dispose()
    {
        vk.DestroyPipeline(device.Device, GraphicsPipeline, null);
        vk.DestroyPipelineLayout(device.Device, PipelineLayout, null);
    }
}
