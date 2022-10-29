struct CameraUniform {
    view_ortho: mat4x4<f32>
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput1 {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput
) -> VertexOutput1 {

    var out: VertexOutput1;
    out.color = vertex.color;
    out.clip_position = camera.view_ortho * vec4<f32>(vertex.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput1) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}