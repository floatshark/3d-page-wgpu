struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1)       uv: vec2<f32>
};

@group(0) @binding(0) var<uniform> inTransform: mat4x4<f32>;
@group(0) @binding(1) var inTexture: texture_2d<f32>;
@group(0) @binding(2) var inSampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) uv      : vec2<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.position = inTransform * position;
    result.uv       = uv;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var tex = textureSample(inTexture, inSampler, vertex.uv);
    return tex;
    //return vec4<f32>(vertex.uv[0], vertex.uv[1], 0, 1);
}