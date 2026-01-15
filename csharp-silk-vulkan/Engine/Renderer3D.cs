namespace Experiment.Engine;

using System.Numerics;
using Experiment.VulkanUtils;
using Silk.NET.Maths;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;
using Vertex = Vertex3DTexturedRgba;

// TODO this is basically the same as 2D, can we generalize?
public sealed unsafe class Renderer3D : IDisposable
{
    private class ModelUniforms : IDisposable
    {
        private readonly Vk vk;

        private readonly Uniforms uniforms;
        private readonly Uniforms.BufferBinding<Matrix4X4<float>> modelMatrixBinding;
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
                        Binding = UNIFORM_MODEL_MATRIX_BINDING,
                        DescriptorCount = UNIFORM_SET_INDEX_MODEL,
                        DescriptorType = DescriptorType.UniformBuffer,
                        PImmutableSamplers = null,
                        StageFlags = ShaderStageFlags.VertexBit,
                    },
                    new()
                    {
                        Binding = UNIFORM_MODEL_SAMPLER_BINDING,
                        DescriptorCount = UNIFORM_SET_INDEX_MODEL,
                        DescriptorType = DescriptorType.CombinedImageSampler,
                        PImmutableSamplers = null,
                        StageFlags = ShaderStageFlags.FragmentBit,
                    },
                ]
            );
            modelMatrixBinding = uniforms.GetBufferBinding<Matrix4X4<float>>(
                UNIFORM_MODEL_MATRIX_BINDING
            );
            samplerBinding = uniforms.GetTextureBinding(UNIFORM_MODEL_SAMPLER_BINDING);
        }

        public void Dispose()
        {
            samplerBinding.Dispose();
            modelMatrixBinding.Dispose();
            uniforms.Dispose();
        }

        public DescriptorSetLayoutWrapper DescriptorSetLayout => uniforms.DescriptorSetLayout;

        public void Bind(
            CommandBufferWrapper commandBuffer,
            GraphicsPipelineWrapper<Vertex> graphicsPipeline,
            Matrix4X4<float> modelMatrix,
            TextureImageWrapper texture
        )
        {
            vk.CmdBindDescriptorSets(
                commandBuffer.CommandBuffer,
                PipelineBindPoint.Graphics,
                graphicsPipeline.PipelineLayout,
                UNIFORM_SET_INDEX_MODEL,
                1,
                in uniforms.DescriptorSet.DescriptorSet,
                0,
                null
            );
            modelMatrixBinding.Value = modelMatrix;
            samplerBinding.Value = texture;
        }
    }

    private const uint UNIFORM_SET_INDEX_SCENE = 0;
    private const uint UNIFORM_SET_INDEX_MODEL = 1;

    private const uint UNIFORM_SCENE_PROJECTION_MATRIX_BINDING = 0;

    private const uint UNIFORM_MODEL_MATRIX_BINDING = 0;
    private const uint UNIFORM_MODEL_SAMPLER_BINDING = 1;

    private readonly Vk vk;
    private readonly Shaderc shaderc;
    private readonly PhysicalDeviceWrapper physicalDevice;
    private readonly DeviceWrapper device;

    private readonly Uniforms uniformsScene;
    private readonly Uniforms.BufferBinding<Matrix4X4<float>> uniformSceneProjectionMatrixBinding;

    private readonly List<ModelUniforms> modelUniformsList;
    private int nextModelUniformsIndex;

    private GraphicsPipelineWrapper<Vertex>? graphicsPipeline;

    public Renderer3D(
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

    public void Render(
        SwapchainWrapper swapchain,
        RenderPassWrapper renderPass,
        CommandBufferWrapper commandBuffer,
        Matrix4X4<float> projectionMatrix,
        Action<Action<Matrix4X4<float>, TextureImageWrapper, Action>> callback
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
            UNIFORM_SET_INDEX_SCENE,
            1,
            in uniformsScene.DescriptorSet.DescriptorSet,
            0,
            null
        );
        uniformSceneProjectionMatrixBinding.Value = projectionMatrix;

        nextModelUniformsIndex = 0;

        callback(
            (modelMatrix, texture, innerCallback) =>
            {
                ModelUniforms modelUniforms;
                if (nextModelUniformsIndex >= modelUniformsList.Count)
                {
                    modelUniforms = new ModelUniforms(vk, physicalDevice, device);
                    modelUniformsList.Add(modelUniforms);
                }
                else
                {
                    modelUniforms = modelUniformsList[nextModelUniformsIndex];
                    nextModelUniformsIndex++;
                }
                modelUniforms.Bind(commandBuffer, graphicsPipeline, modelMatrix, texture);
                innerCallback();
            }
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

            layout(set = {{UNIFORM_SET_INDEX_SCENE}}, binding = {{UNIFORM_SCENE_PROJECTION_MATRIX_BINDING}}) uniform UniformScene {
                mat4 projection;
            } uniformScene;

            layout(set = {{UNIFORM_SET_INDEX_MODEL}}, binding = {{UNIFORM_MODEL_MATRIX_BINDING}}) uniform UniformModel {
                mat4 model;
            } uniformModel;

            layout(location = {{Vertex.POSITION_LOCATION}}) in vec3 inPosition;
            layout(location = {{Vertex.TEXTURE_COORDINATE_LOCATION}}) in vec2 inTextureCoordinate;
            layout(location = {{Vertex.COLOR_LOCATION}}) in vec4 inColor;

            layout(location = 0) out vec2 fragTextureCoordinate;
            layout(location = 1) out vec4 fragColor;

            void main() {
                gl_Position = uniformScene.projection * uniformModel.model * vec4(inPosition, 1.0);
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

            layout(set = {{UNIFORM_SET_INDEX_MODEL}}, binding = {{UNIFORM_MODEL_SAMPLER_BINDING}}) uniform sampler2D uniformSampler;

            layout(location = 0) in vec2 fragTextureCoordinate;
            layout(location = 1) in vec4 fragColor;

            layout(location = 0) out vec4 outColor;

            void main() {
                outColor = texture(uniformSampler, fragTextureCoordinate) * fragColor;
            }
            """
        );

        using var referenceExampleModelUniforms = new ModelUniforms(vk, physicalDevice, device);

        graphicsPipeline?.Dispose();
        graphicsPipeline = new(
            vk,
            device,
            swapchain,
            renderPass,
            vertexShaderModule,
            fragmentShaderModule,
            [
                // order matters here, see the constants for sets, UNIFORM_SET_INDEX_SCENE and UNIFORM_SET_INDEX_MODEL
                uniformsScene.DescriptorSetLayout,
                referenceExampleModelUniforms.DescriptorSetLayout,
            ],
            // TODO configurable blending
            false
        );
        return graphicsPipeline;
    }
}
