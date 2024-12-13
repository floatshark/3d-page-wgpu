struct VertexOutput {
    @builtin(position) position: vec4<f32>,
	@location(0)       normal  : vec3<f32>,
};

struct FragmentOutput {
    @location(0) position : vec4<f32>,
    @location(1) normal   : vec4<f32>,
}

@group(0) @binding(0) var<uniform> inTransform: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) position : vec4<f32>,
    @location(1) normal   : vec3<f32>
) -> VertexOutput 
{
    var output : VertexOutput;

    output.position = inTransform * position;
    output.normal   = normal;

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