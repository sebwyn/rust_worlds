struct CameraUniform {
    view_ortho: mat4x4<f32>
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct CameraUniform {
    view_ortho: mat4x4<f32>
}


struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOutput1 {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput1 {

    var out: VertexOutput1;

    out.tex_coord = vertex.tex_coord;
    out.color = vertex.color;
    out.clip_position = camera.view_ortho * vec4<f32>(vertex.position, 0.0, 1.0);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput1) -> @location(0) vec4<f32> {
    return  textureSample(t_diffuse, s_diffuse, in.tex_coord) * vec4<f32>(in.color, 1.0);
}