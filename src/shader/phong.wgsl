struct VertexOutput 
{
    @builtin(position) position: vec4<f32>,
    @location(0)       normal  : vec3<f32>,
};

struct Uniform 
{
    transform_matrix   : mat4x4<f32>,
    rotation_matrix    : mat4x4<f32>,
    directional_light  : vec4<f32>,
    ambient_light      : vec4<f32>,
    inverse_matrix     : mat4x4<f32>,
    buffer_type        : f32,
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
    result.normal   = (inUniform.rotation_matrix * vec4<f32>(normal, 1.0)).xyz;
    return result;
}

@fragment
fn fs_main(
    vertex: VertexOutput
) -> @location(0) vec4<f32> 
{
    let directional_light : vec3<f32> = normalize(inUniform.directional_light.xyz);
    let normal            : vec3<f32> = normalize(vertex.normal);
    let diffuse           : f32       = max(dot(-1.0 * directional_light, normal), 0.0);

    let view     : vec3<f32> = normalize((inUniform.inverse_matrix * vertex.position).xyz);
    let halfway  : vec3<f32> = -normalize(directional_light.xyz + view);
    let specular : f32       = pow(max(dot(normal, halfway), 0.0), 100.0);

    let ambient_light     : vec4<f32> = inUniform.ambient_light;

    let surface_color  : vec4<f32> = vec4(0.5, 0.5, 0.5, 1.0);
    let specular_color : vec4<f32> = vec4(1.0, 1.0, 1.0, 1.0);

    var frag_color = diffuse * surface_color + specular * surface_color + ambient_light;

    // ummm
    if(inUniform.buffer_type == 1.0)
    {
      return vec4((normal / 2.0 + 0.5), 1.0);
    }

    return frag_color;
}