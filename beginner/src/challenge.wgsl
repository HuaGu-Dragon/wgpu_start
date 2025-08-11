struct VertexInput {
    @location(0) clip_position: vec3f,
    @location(1) color: vec4f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4f(model.clip_position, 1.0);
    return out;
}

// 片元着色器

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color);
}
