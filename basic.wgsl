// Vertex shader
@group(0) @binding(0) var<uniform> tex_offset: vec2<f32>;

struct VertexInput {
    @location(0)  position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(
    in: VertexInput
) -> VertexOutput {

    /*
    var tex_coord = vec2<f32>(0.0, 0.0);
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    if in_vertex_index == u32(0) {
        tex_coord = vec2<f32>(0.8, 0.2);
    } else if in_vertex_index == u32(1) {
        tex_coord = vec2<f32>(0.5, 0.8);
    } else if in_vertex_index == u32(2) {
        tex_coord = vec2<f32>(0.2, 0.2);
    }
    tex_coord += tex_offset;
    out.tex_coord = clamp(tex_coord, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    */

    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coord = in.tex_coord; 

    return out;
}

// Fragment shader
@group(1) @binding(0) var diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse, s_diffuse, in.tex_coord);
}
