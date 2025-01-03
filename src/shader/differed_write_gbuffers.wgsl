struct VertexOutput {
    @builtin(position) position   : vec4<f32>,
	@location(0)       normal     : vec3<f32>,
    @location(1)       uv         : vec2<f32>,
    @location(2)       tangent    : vec3<f32>,
};

struct FragmentOutput {
    @location(0) position : vec4<f32>,
    @location(1) normal   : vec4<f32>,
    @location(2) albedo   : vec4<f32>,
}

struct Uniform {
    model_matrix      : mat4x4<f32>,
    view_matrix       : mat4x4<f32>,
    projection_matrix : mat4x4<f32>,
    rotation_matrix   : mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> inUniform : Uniform;
@group(1) @binding(0) var base_color_texture : texture_2d<f32>;
@group(1) @binding(1) var base_color_sampler : sampler;
@group(1) @binding(2) var normal_texture     : texture_2d<f32>;
@group(1) @binding(3) var normal_sampler     : sampler;

@vertex
fn vs_main(
    @location(0) position : vec4<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) uv       : vec2<f32>,
) -> VertexOutput 
{
    let normal_world   = normalize(inUniform.rotation_matrix * vec4<f32>(normal, 1.0)).xyz;
	let tangent_world  = normalize(inUniform.rotation_matrix * vec4<f32>(0.0, 1.0, 0.0, 1.0)).xyz;

    var output : VertexOutput;

    output.position  = inUniform.projection_matrix * inUniform.view_matrix * inUniform.model_matrix * position;
    output.normal    = normal_world;
    output.uv        = uv;
    output.tangent   = tangent_world;

    return output;
}

@fragment
fn fs_main(vertex: VertexOutput) -> FragmentOutput 
{
	let binormal_world = normalize(cross(vertex.normal, vertex.tangent));
	let tbn_matrix     = mat3x3<f32>(vertex.tangent, binormal_world, vertex.normal);

	var output : FragmentOutput;

    output.position = vertex.position;
    output.normal   = vec4<f32>(tbn_matrix * textureSample(normal_texture, normal_sampler, vertex.uv).xyz, 1.0);
    output.albedo   = textureSample(base_color_texture, base_color_sampler, vertex.uv);

    return output;
}