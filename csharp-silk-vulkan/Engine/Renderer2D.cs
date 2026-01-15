namespace Experiment.Engine;

using System.Numerics;
using Experiment.VulkanUtils;
using Silk.NET.Maths;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;
using Vertex = Vertex2DTexturedRgba;

public sealed unsafe class Renderer2D : IDisposable
{
    private const uint UNIFORM_SCENE_BINDING = 0;
    private const uint UNIFORM_SAMPLER_BINDING = 1;

    private readonly Vk vk;
    private readonly Shaderc shaderc;
    private readonly DeviceWrapper device;

    private readonly Uniforms uniforms;
    private readonly Uniforms.BufferBinding<Matrix4X4<float>> uniformSceneBinding;
    private readonly Uniforms.TextureBinding uniformSamplerBinding;

    private GraphicsPipelineWrapper<Vertex>? graphicsPipeline;

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
                    Binding = UNIFORM_SCENE_BINDING,
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

        uniformSceneBinding = uniforms.GetBufferBinding<Matrix4X4<float>>(UNIFORM_SCENE_BINDING);

        uniformSamplerBinding = uniforms.GetTextureBinding(UNIFORM_SAMPLER_BINDING);

        ProjectionMatrix = Matrix4X4<float>.Identity;
    }

    public void Dispose()
    {
        graphicsPipeline?.Dispose();
        uniformSamplerBinding.Dispose();
        uniformSceneBinding.Dispose();
        uniforms.Dispose();
    }

    public Matrix4X4<float> ProjectionMatrix
    {
        get => uniformSceneBinding.Value;
        set => uniformSceneBinding.Value = value;
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

    private GraphicsPipelineWrapper<Vertex> CreateGraphicsPipelineIfNeeded(
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
            $$"""
            #version 450

            layout(binding = {{UNIFORM_SCENE_BINDING}}) uniform UniformScene {
                mat4 projection;
            } uniformScene;

            // TODO uniformModel, with a model matrix

            layout(location = {{Vertex.POSITION_LOCATION}}) in vec2 inPosition;
            layout(location = {{Vertex.TEXTURE_COORDINATE_LOCATION}}) in vec2 inTextureCoordinate;
            layout(location = {{Vertex.COLOR_LOCATION}}) in vec4 inColor;

            layout(location = 0) out vec2 fragTextureCoordinate;
            layout(location = 1) out vec4 fragColor;

            void main() {
                gl_Position = uniformScene.projection * vec4(inPosition, 0.0, 1.0);
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
            $$"""
            #version 450

            layout(binding = {{UNIFORM_SAMPLER_BINDING}}) uniform sampler2D uniformSampler;

            layout(location = 0) in vec2 fragTextureCoordinate;
            layout(location = 1) in vec4 fragColor;

            layout(location = 0) out vec4 outColor;

            void main() {
                outColor = texture(uniformSampler, fragTextureCoordinate) * fragColor;
            }
            """
        );

        graphicsPipeline?.Dispose();
        graphicsPipeline = new(
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
