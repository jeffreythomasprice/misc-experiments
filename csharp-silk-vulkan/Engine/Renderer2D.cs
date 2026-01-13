namespace Experiment.Engine;

using System.Numerics;
using Experiment.VulkanUtils;
using Silk.NET.Maths;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;

public sealed unsafe class Renderer2D : IDisposable
{
    private struct UniformMatrices
    {
        public Matrix4X4<float> Model;
        public Matrix4X4<float> View;
        public Matrix4X4<float> Projection;
    }

    private const uint UNIFORM_MATRICES_BINDING = 0;
    private const uint UNIFORM_SAMPLER_BINDING = 1;

    private readonly Vk vk;
    private readonly Shaderc shaderc;
    private readonly DeviceWrapper device;

    private readonly Uniforms uniforms;
    private readonly Uniforms.BufferBinding<UniformMatrices> uniformMatricesBinding;
    private readonly Uniforms.TextureBinding uniformSamplerBinding;

    private GraphicsPipelineWrapper<Vertex2DTexturedRgba>? graphicsPipeline;

    public Renderer2D(
        Vk vk,
        Shaderc shaderc,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device
    )
    {
        this.vk = vk;
        this.shaderc = shaderc;
        this.device = device;

        uniforms = new(
            vk,
            physicalDevice,
            device,
            [
                new()
                {
                    Binding = UNIFORM_MATRICES_BINDING,
                    DescriptorCount = 1,
                    DescriptorType = DescriptorType.UniformBuffer,
                    PImmutableSamplers = null,
                    StageFlags = ShaderStageFlags.VertexBit,
                },
                new()
                {
                    Binding = UNIFORM_SAMPLER_BINDING,
                    DescriptorCount = 1,
                    DescriptorType = DescriptorType.CombinedImageSampler,
                    PImmutableSamplers = null,
                    StageFlags = ShaderStageFlags.FragmentBit,
                },
            ]
        );

        uniformMatricesBinding = uniforms.GetBufferBinding<UniformMatrices>(
            UNIFORM_MATRICES_BINDING
        );

        uniformSamplerBinding = uniforms.GetTextureBinding(UNIFORM_SAMPLER_BINDING);

        ModelMatrix = Matrix4X4<float>.Identity;
        ViewMatrix = Matrix4X4<float>.Identity;
        ProjectionMatrix = Matrix4X4<float>.Identity;
    }

    public void Dispose()
    {
        graphicsPipeline?.Dispose();
        uniformSamplerBinding.Dispose();
        uniformMatricesBinding.Dispose();
        uniforms.Dispose();
    }

    public Matrix4X4<float> ModelMatrix
    {
        get => Matrices.Model;
        set => Matrices = Matrices with { Model = value };
    }

    public Matrix4X4<float> ViewMatrix
    {
        get => Matrices.View;
        set => Matrices = Matrices with { View = value };
    }

    public Matrix4X4<float> ProjectionMatrix
    {
        get => Matrices.Projection;
        set => Matrices = Matrices with { Projection = value };
    }

    public TextureImageWrapper? Texture
    {
        get => uniformSamplerBinding.Value;
        set => uniformSamplerBinding.Value = value;
    }

    public void Bind(
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        CommandBufferWrapper commandBuffer
    )
    {
        var graphicsPipeline = CreateGraphicsPipelineIfNeeded(swapchain, renderPass);

        vk.CmdBindPipeline(
            commandBuffer.CommandBuffer,
            PipelineBindPoint.Graphics,
            graphicsPipeline.GraphicsPipeline
        );

        vk.CmdBindDescriptorSets(
            commandBuffer.CommandBuffer,
            PipelineBindPoint.Graphics,
            graphicsPipeline.PipelineLayout,
            0,
            1,
            in uniforms.DescriptorSet.DescriptorSet,
            0,
            null
        );
    }

    public void OnSwapchainDestroyed()
    {
        graphicsPipeline?.Dispose();
        graphicsPipeline = null;
    }

    private UniformMatrices Matrices
    {
        get => uniformMatricesBinding.Value;
        set => uniformMatricesBinding.Value = value;
    }

    private GraphicsPipelineWrapper<Vertex2DTexturedRgba> CreateGraphicsPipelineIfNeeded(
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass
    )
    {
        if (graphicsPipeline != null)
        {
            return graphicsPipeline;
        }

        using var vertexShaderModule = ShaderModuleWrapper.FromGlslSource(
            vk,
            shaderc,
            device,
            ShaderModuleWrapper.ShaderType.Vertex,
            """
            #version 450

            layout(binding = 0) uniform UniformMatrices {
                mat4 model;
                mat4 view;
                mat4 projection;
            } uniformMatrices;

            layout(location = 0) in vec2 inPosition;
            layout(location = 1) in vec2 inTextureCoordinate;
            layout(location = 2) in vec4 inColor;

            layout(location = 0) out vec2 fragTextureCoordinate;
            layout(location = 1) out vec4 fragColor;

            void main() {
                gl_Position = uniformMatrices.projection * uniformMatrices.view * uniformMatrices.model * vec4(inPosition, 0.0, 1.0);
                fragTextureCoordinate = inTextureCoordinate;
                fragColor = inColor;
            }
            """
        );

        using var fragmentShaderModule = ShaderModuleWrapper.FromGlslSource(
            vk,
            shaderc,
            device,
            ShaderModuleWrapper.ShaderType.Fragment,
            """
            #version 450

            layout(binding = 1) uniform sampler2D uniformSampler;

            layout(location = 0) in vec2 fragTextureCoordinate;
            layout(location = 1) in vec4 fragColor;

            layout(location = 0) out vec4 outColor;

            void main() {
                outColor = texture(uniformSampler, fragTextureCoordinate) * fragColor;
            }
            """
        );

        graphicsPipeline?.Dispose();
        graphicsPipeline = new GraphicsPipelineWrapper<Vertex2DTexturedRgba>(
            vk,
            device,
            swapchain,
            renderPass,
            vertexShaderModule,
            fragmentShaderModule,
            [uniforms.DescriptorSetLayout]
        );
        return graphicsPipeline;
    }
}
