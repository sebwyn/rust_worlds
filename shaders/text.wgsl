// Vertex shader
struct VertexInput {
    @location(0)  position: vec2<f32>,
    @location(1)  tex_coord: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct InstanceInput {
    @location(2) model_matrix_0: vec3<f32>,
    @location(3) model_matrix_1: vec3<f32>,
    @location(4) model_matrix_2: vec3<f32>,

    @location(5) tex_matrix_0: vec3<f32>,
    @location(6) tex_matrix_1: vec3<f32>,
    @location(7) tex_matrix_2: vec3<f32>,
};

@group(0) @binding(0) var<uniform> ortho_matrix: mat4x4<f32>;

@vertex
fn vs_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat3x3<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
    );

    let tex_matrix = mat3x3<f32>(
        instance.tex_matrix_0,
        instance.tex_matrix_1,
        instance.tex_matrix_2,
    );
    
    out.clip_position = ortho_matrix *  vec4<f32>(model_matrix * vec3<f32>(in.position, 1.0), 1.0);
    out.clip_position.z = 0.0;

    out.tex_coord = (tex_matrix * vec3<f32>(in.tex_coord, 1.0)).xy;

    return out;
}

// Fragment shader
@group(1) @binding(0) var font: texture_2d<f32>;
@group(1) @binding(1) var s_font: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(font, s_font, in.tex_coord);
}