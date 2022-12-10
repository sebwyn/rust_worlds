// Vertex shader
struct VertexInput {
    @location(0)  position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0) var<uniform> combined_matrix: mat4x4<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {

    var out: VertexOutput;
    out.clip_position = combined_matrix * vec4<f32>(in.position, 1.0);

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}
