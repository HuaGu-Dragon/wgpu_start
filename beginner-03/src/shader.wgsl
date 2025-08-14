struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coord: vec2f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coord: vec2f,
};

@vertex
fn vs_main(
    input: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4f(input.position, 1.0);
    out.tex_coord = input.tex_coord;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(t_diffuse, s_diffuse, in.tex_coord);
}