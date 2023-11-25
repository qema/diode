struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> size: vec2<f32>;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.uv = uv;
    result.color = color;
    result.position = vec4<f32>(
        (position.x / size.x) * 2.0 - 1.0,
        1.0 - (position.y / size.y) * 2.0,
        0.0, 1.0);
    return result;
}

@group(0) @binding(1)
var tex: texture_2d<f32>;
@group(0) @binding(2)
var samp: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, vertex.uv) * vertex.color;
    //return vertex.color;
}
