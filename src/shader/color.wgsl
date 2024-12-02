struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1)       color   : vec3<f32>
};

@group(0) @binding(0) var<uniform> inTransform: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) color   : vec3<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.position = inTransform * position;
    result.color    = color;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(vertex.color.x, vertex.color.y, vertex.color.z, 1.0);
}