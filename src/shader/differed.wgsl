@vertex
fn vs_main( @builtin(vertex_index) VertexIndex : u32 ) -> @builtin(position) vec4f 
{
  const pos = array(
    vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
    vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
  );

  return vec4f(pos[VertexIndex], 0.0, 1.0);
}

struct Uniform
{
    directional_light  : vec4<f32>,
    ambient_light      : vec4<f32>,
    inverse_matrix     : mat4x4<f32>,
    buffer_type        : f32,
}

@group(0) @binding(0) var gbuffer_position : texture_2d<f32>;
@group(0) @binding(1) var gbuffer_normal   : texture_2d<f32>;
@group(0) @binding(2) var gbuffer_depth    : texture_depth_2d;
@group(0) @binding(3) var gbuffer_albedo   : texture_2d<f32>;
@group(1) @binding(0) var<uniform> inUniform: Uniform;

@fragment
fn fs_main( @builtin(position) coord : vec4f ) -> @location(0) vec4f
{
    let position : vec4f     = textureLoad( gbuffer_position, vec2i(floor(coord.xy)), 0 );
    var normal   : vec3<f32> = textureLoad( gbuffer_normal, vec2i(floor(coord.xy)), 0 ).xyz;
    var depth    : f32       = textureLoad( gbuffer_depth, vec2i(floor(coord.xy)), 0 );
    var albedo   : vec4<f32> = textureLoad( gbuffer_albedo, vec2i(floor(coord.xy)), 0 );

    if (depth >= 1.0) 
    {
      discard;
    }

    let directional_light : vec3<f32> = normalize(inUniform.directional_light.xyz);
    let diffuse           : f32       = max(dot(-1.0 * directional_light, normal), 0.0);

    let view     : vec3<f32> = normalize((inUniform.inverse_matrix * position).xyz);
    let halfway  : vec3<f32> = -normalize(directional_light.xyz + view);
    let specular : f32       = pow(max(dot(normal, halfway), 0.0), 100.0);

    let ambient_light     : vec4<f32> = inUniform.ambient_light;

    let surface_color  : vec4<f32> = albedo;
    let specular_color : vec4<f32> = vec4(1.0, 1.0, 1.0, 1.0);

    var frag_color = diffuse * surface_color + specular * surface_color + ambient_light;
    return frag_color;
}

@fragment
fn fs_debug_main( @builtin(position) coord : vec4f ) -> @location(0) vec4f
{
    let position : vec4f     = textureLoad( gbuffer_position, vec2i(floor(coord.xy)), 0 );
    var normal   : vec3<f32> = textureLoad( gbuffer_normal, vec2i(floor(coord.xy)), 0 ).xyz;
    var depth    : f32       = textureLoad( gbuffer_depth, vec2i(floor(coord.xy)), 0 );
    let albedo   : vec4<f32> = textureLoad( gbuffer_albedo, vec2i(floor(coord.xy)), 0 );

    normal.x = (normal.x + 1.0) * 0.5;
    normal.y = (normal.y + 1.0) * 0.5;
    normal.z = (normal.z + 1.0) * 0.5;

    depth = (1.0 - depth) * 50.0;

    // ummm
    if(inUniform.buffer_type == 1.0)
    {
      return vec4(normal, 1.0);
    }
    else if(inUniform.buffer_type == 2.0)
    {
      return vec4(depth, 0.0, 0.0, 1.0);
    }
    else if(inUniform.buffer_type == 3.0)
    {
      return albedo;
    }

    return vec4(depth, 0.0, 0.0, 1.0);
}