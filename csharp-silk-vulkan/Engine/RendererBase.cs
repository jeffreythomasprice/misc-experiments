namespace Experiment.Engine;

using Experiment.VulkanUtils;
using Silk.NET.Maths;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;

public abstract unsafe class RendererBase<VertexType> : IDisposable
    where VertexType : IBufferBindable
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
            GraphicsPipelineWrapper<VertexType> graphicsPipeline,
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

    protected const uint UNIFORM_SET_INDEX_SCENE = 0;
    protected const uint UNIFORM_SET_INDEX_MODEL = 1;

    protected const uint UNIFORM_SCENE_PROJECTION_MATRIX_BINDING = 0;

    protected const uint UNIFORM_MODEL_MATRIX_BINDING = 0;
    protected const uint UNIFORM_MODEL_SAMPLER_BINDING = 1;

    private readonly Vk vk;
    private readonly Shaderc shaderc;
    private readonly PhysicalDeviceWrapper physicalDevice;
    private readonly DeviceWrapper device;
    private readonly GraphicsPipelineWrapper<VertexType>.Options graphicsPipelineOptions;

    private readonly Uniforms uniformsScene;
    private readonly Uniforms.BufferBinding<Matrix4X4<float>> uniformSceneProjectionMatrixBinding;

    private readonly List<ModelUniforms> modelUniformsList;
    private int nextModelUniformsIndex;

    private GraphicsPipelineWrapper<VertexType>? graphicsPipeline;

    public RendererBase(
        Vk vk,
        Shaderc shaderc,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        GraphicsPipelineWrapper<VertexType>.Options graphicsPipelineOptions
    )
    {
        this.vk = vk;
        this.shaderc = shaderc;
        this.physicalDevice = physicalDevice;
        this.device = device;
        this.graphicsPipelineOptions = graphicsPipelineOptions;

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

    private GraphicsPipelineWrapper<VertexType> CreateGraphicsPipelineIfNeeded(
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
            VertexShaderSource
        );

        using var fragmentShaderModule = ShaderModuleWrapper.FromGlslSource(
            vk,
            shaderc,
            device,
            ShaderModuleWrapper.ShaderType.Fragment,
            FragmentShaderSource
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
            graphicsPipelineOptions
        );
        return graphicsPipeline;
    }

    protected abstract string VertexShaderSource { get; }
    protected abstract string FragmentShaderSource { get; }
}
