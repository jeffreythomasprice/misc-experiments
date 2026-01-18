namespace Experiment.Engine;

using Experiment.VulkanUtils;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;
using VertexType = Vertex2DTexturedRgba;

public sealed class Renderer2D : RendererBase<VertexType>
{
    public Renderer2D(
        Vk vk,
        Shaderc shaderc,
        PhysicalDeviceWrapper physicalDevice,
        DeviceWrapper device,
        GraphicsPipelineWrapper<VertexType>.Options graphicsPipelineOptions
    )
        : base(vk, shaderc, physicalDevice, device, graphicsPipelineOptions) { }

    protected override string VertexShaderSource =>
        $$"""
            #version 450

            layout(set = {{UNIFORM_SET_INDEX_SCENE}}, binding = {{UNIFORM_SCENE_PROJECTION_MATRIX_BINDING}}) uniform UniformScene {
                mat4 projection;
            } uniformScene;

            layout(set = {{UNIFORM_SET_INDEX_MODEL}}, binding = {{UNIFORM_MODEL_MATRIX_BINDING}}) uniform UniformModel {
                mat4 model;
            } uniformModel;

            layout(location = {{VertexType.POSITION_LOCATION}}) in vec2 inPosition;
            layout(location = {{VertexType.TEXTURE_COORDINATE_LOCATION}}) in vec2 inTextureCoordinate;
            layout(location = {{VertexType.COLOR_LOCATION}}) in vec4 inColor;

            layout(location = 0) out vec2 fragTextureCoordinate;
            layout(location = 1) out vec4 fragColor;

            void main() {
                gl_Position = uniformScene.projection * uniformModel.model * vec4(inPosition, 0.0, 1.0);
                fragTextureCoordinate = inTextureCoordinate;
                fragColor = inColor;
            }
            """;

    protected override string FragmentShaderSource =>
        $$"""
            #version 450

            layout(set = {{UNIFORM_SET_INDEX_MODEL}}, binding = {{UNIFORM_MODEL_SAMPLER_BINDING}}) uniform sampler2D uniformSampler;

            layout(location = 0) in vec2 fragTextureCoordinate;
            layout(location = 1) in vec4 fragColor;

            layout(location = 0) out vec4 outColor;

            void main() {
                outColor = texture(uniformSampler, fragTextureCoordinate) * fragColor;
            }
            """;
}
