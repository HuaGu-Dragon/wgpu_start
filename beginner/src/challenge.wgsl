struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    if (in_vertex_index == 0) {
        out.color = vec3f(1.0, 0.0, 0.0);
    } else if (in_vertex_index == 1) {
        out.color = vec3f(0.0, 1.0, 0.0);
    } else {
        out.color = vec3f(0.0, 0.0, 1.0);
    }
    out.clip_position = vec4f(x, y, 0.0, 1.0);
    return out;
}

// 片元着色器

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
