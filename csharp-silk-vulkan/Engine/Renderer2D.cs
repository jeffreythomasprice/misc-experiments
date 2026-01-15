namespace Experiment.Engine;

using System.Numerics;
using Experiment.VulkanUtils;
using Silk.NET.Maths;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;
using Vertex = Vertex2DTexturedRgba;

public sealed unsafe class Renderer2D : IDisposable
{
    private class ModelUniforms : IDisposable
    {
        private readonly Vk vk;

        private readonly Uniforms uniforms;
        private readonly Uniforms.TextureBinding samplerBinding;

        public ModelUniforms(Vk vk, PhysicalDeviceWrapper physicalDevice, DeviceWrapper device)
        {
            this.vk = vk;

            uniforms = new(
                vk,
                physicalDevice,
                device,
                [
                    new()
                    {
                        Binding = UNIFORM_MODEL_SAMPLER_BINDING,
                        DescriptorCount = 1,
                        DescriptorType = DescriptorType.CombinedImageSampler,
                        PImmutableSamplers = null,
                        StageFlags = ShaderStageFlags.FragmentBit,
                    },
                ]
            );
            samplerBinding = uniforms.GetTextureBinding(UNIFORM_MODEL_SAMPLER_BINDING);
        }

        public void Dispose()
        {
            samplerBinding.Dispose();
            uniforms.Dispose();
        }

        public DescriptorSetLayoutWrapper DescriptorSetLayout => uniforms.DescriptorSetLayout;

        public void Bind(
            CommandBufferWrapper commandBuffer,
            GraphicsPipelineWrapper<Vertex> graphicsPipeline,
            TextureImageWrapper texture
        )
        {
            vk.CmdBindDescriptorSets(
                commandBuffer.CommandBuffer,
                PipelineBindPoint.Graphics,
                graphicsPipeline.PipelineLayout,
                // TODO the fact that this is set 1 should be a constant?
                1,
                1,
                in uniforms.DescriptorSet.DescriptorSet,
                0,
                null
            );
            samplerBinding.Value = texture;
        }
    }

    // set 0, scene
    private const uint UNIFORM_SCENE_PROJECTION_MATRIX_BINDING = 0;

    // set 1, model
    // TODO UNIFORM_SCENE_MODEL_MATRIX_BINDING
    private const uint UNIFORM_MODEL_SAMPLER_BINDING = 0;

    private readonly Vk vk;
    private readonly Shaderc shaderc;
    private readonly PhysicalDeviceWrapper physicalDevice;
    private readonly DeviceWrapper device;

    private readonly Uniforms uniformsScene;
    private readonly Uniforms.BufferBinding<Matrix4X4<float>> uniformSceneProjectionMatrixBinding;

    private readonly List<ModelUniforms> modelUniformsList;
    private int nextModelUniformsIndex;

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
        this.physicalDevice = physicalDevice;
        this.device = device;

        // TODO multiple uniforms should share a descriptor pool?

        uniformsScene = new(
            vk,
            physicalDevice,
            device,
            [
                new()
                {
                    Binding = UNIFORM_SCENE_PROJECTION_MATRIX_BINDING,
                    DescriptorCount = 1,
                    DescriptorType = DescriptorType.UniformBuffer,
                    PImmutableSamplers = null,
                    StageFlags = ShaderStageFlags.VertexBit,
                },
            ]
        );
        uniformSceneProjectionMatrixBinding = uniformsScene.GetBufferBinding<Matrix4X4<float>>(
            UNIFORM_SCENE_PROJECTION_MATRIX_BINDING
        );

        modelUniformsList = [];
    }

    public void Dispose()
    {
        graphicsPipeline?.Dispose();
        foreach (var x in modelUniformsList)
        {
            x.Dispose();
        }
        uniformSceneProjectionMatrixBinding.Dispose();
        uniformsScene.Dispose();
    }

    public void Bind(
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        CommandBufferWrapper commandBuffer,
        Matrix4X4<float> projectionMatrix
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
            // TODO the fact that this is set 0 should be a constant?
            0,
            1,
            in uniformsScene.DescriptorSet.DescriptorSet,
            0,
            null
        );
        uniformSceneProjectionMatrixBinding.Value = projectionMatrix;

        ResetModelUniforms();
    }

    // TODO this api is weird, maybe a callback thing in Bind, refactor to BindAndRender
    public void NextModelUniforms(
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        CommandBufferWrapper commandBuffer,
        TextureImageWrapper texture
    )
    {
        var graphicsPipeline = CreateGraphicsPipelineIfNeeded(swapchain, renderPass);

        GetNextModelUniforms().Bind(commandBuffer, graphicsPipeline, texture);
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

            // TODO can set = 0 be a constant?
            layout(set = 0, binding = {{UNIFORM_SCENE_PROJECTION_MATRIX_BINDING}}) uniform UniformScene {
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

            // TODO can set = 1 be a constant?
            layout(set = 1, binding = {{UNIFORM_MODEL_SAMPLER_BINDING}}) uniform sampler2D uniformSampler;

            layout(location = 0) in vec2 fragTextureCoordinate;
            layout(location = 1) in vec4 fragColor;

            layout(location = 0) out vec4 outColor;

            void main() {
                outColor = texture(uniformSampler, fragTextureCoordinate) * fragColor;
            }
            """
        );

        var referenceExampleModelUniforms = GetNextModelUniforms();

        graphicsPipeline?.Dispose();
        graphicsPipeline = new(
            vk,
            device,
            swapchain,
            renderPass,
            vertexShaderModule,
            fragmentShaderModule,
            [uniformsScene.DescriptorSetLayout, referenceExampleModelUniforms.DescriptorSetLayout]
        );
        return graphicsPipeline;
    }

    private void ResetModelUniforms()
    {
        nextModelUniformsIndex = 0;
    }

    private ModelUniforms GetNextModelUniforms()
    {
        if (nextModelUniformsIndex >= modelUniformsList.Count)
        {
            var result = new ModelUniforms(vk, physicalDevice, device);
            modelUniformsList.Add(result);
            return result;
        }
        else
        {
            var result = modelUniformsList[nextModelUniformsIndex];
            nextModelUniformsIndex++;
            return result;
        }
    }
}
