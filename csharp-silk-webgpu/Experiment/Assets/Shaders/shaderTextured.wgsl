struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) textureCoordinate: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
	@location(0) textureCoordinate: vec2<f32>,
	@location(1) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> projectionMatrixUniform: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> modelviewMatrixUniform: mat4x4<f32>;

@vertex
fn vs_main(
	model: VertexInput,
) -> VertexOutput
{
	var out: VertexOutput;
    out.clip_position = projectionMatrixUniform * modelviewMatrixUniform * vec4<f32>(model.position, 0.0, 1.0);
	out.textureCoordinate = model.textureCoordinate;
    out.color = model.color;
    return out;
}

@group(2) @binding(0)
var diffuseTexture: texture_2d<f32>;
@group(2) @binding(1)
var diffuseSampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f 
{
    return textureSample(diffuseTexture, diffuseSampler, in.textureCoordinate) * in.color;
}