struct VertexOutput 
{
    @builtin(position) position: vec4<f32>,
    @location(0)       normal  : vec3<f32>,
};

struct Uniform 
{
    transform_matrix  : mat4x4<f32>,
    directional_light : vec3<f32>,
    ambient_light     : vec4<f32>,
}

@group(0) @binding(0) var<uniform> inUniform: Uniform;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal  : vec3<f32>,
) -> VertexOutput 
{
    var result: VertexOutput;
    result.position = inUniform.transform_matrix * position;
    result.normal   = normal;
    return result;
}

@fragment
fn fs_main(
    vertex: VertexOutput
) -> @location(0) vec4<f32> 
{
    var directional_light : vec3<f32> = normalize(inUniform.directional_light);
    var normal            : vec3<f32> = normalize(vertex.normal);
    var diffuse           : f32       = max(dot(-1.0 * directional_light, normal), 0.0);

    var ambient_light     : vec4<f32> = inUniform.ambient_light;

    var frag_color = diffuse * vec4(0.5, 0.5, 0.5, 1.0) + ambient_light;
    return frag_color;
}