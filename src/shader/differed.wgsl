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
    inverse_matrix     : mat4x4<f32>
}

@group(0) @binding(0) var gbuffer_position : texture_2d<f32>;
@group(0) @binding(1) var gbuffer_normal   : texture_2d<f32>;
@group(0) @binding(2) var gbuffer_depth    : texture_depth_2d;

@group(1) @binding(0) var<uniform> inUniform: Uniform;

@fragment
fn fs_main( @builtin(position) coord : vec4f ) -> @location(0) vec4f
{
    let position : vec4f = textureLoad( gbuffer_position, vec2i(floor(coord.xy)), 0 );
    var normal   : vec4f = textureLoad( gbuffer_normal, vec2i(floor(coord.xy)), 0 );
    var depth    : f32   = textureLoad( gbuffer_depth, vec2i(floor(coord.xy)), 0 );

    normal.x = (normal.x + 1.0) * 0.5;
    normal.y = (normal.y + 1.0) * 0.5;
    normal.z = (normal.z + 1.0) * 0.5;

    depth = (1.0 - depth) * 50.0;

    //return position;
    //return normal;
    return vec4(depth, 0.0, 0.0, 1.0);
}