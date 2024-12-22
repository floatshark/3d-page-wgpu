struct VertexOutput {
    @builtin(position) position: vec4<f32>,
	@location(0)       normal  : vec3<f32>,
};

struct FragmentOutput {
    @location(0) position : vec4<f32>,
    @location(1) normal   : vec4<f32>,
}

struct Uniform {
    model_matrix      : mat4x4<f32>,
    view_matrix       : mat4x4<f32>,
    projection_matrix : mat4x4<f32>,
    rotation_matrix   : mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> inUniform : Uniform;

@vertex
fn vs_main(
    @location(0) position : vec4<f32>,
    @location(1) normal   : vec3<f32>
) -> VertexOutput 
{
    var output : VertexOutput;

    output.position = inUniform.projection_matrix * inUniform.view_matrix * inUniform.model_matrix * position;
    output.normal   = (inUniform.rotation_matrix * vec4(normal, 1.0)).xyz;

    return output;
}

@fragment
fn fs_main(vertex: VertexOutput) -> FragmentOutput 
{
	var output : FragmentOutput;

    output.position = vertex.position;
    output.normal   = vec4(normalize(vertex.normal), 1.0);

    return output;
}