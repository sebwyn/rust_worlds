// Vertex shader
struct VertexInput {
    @location(0)  position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>, 
    @location(1) rect_position: vec2<f32>,
    @location(2) scale: vec2<f32>,
};

struct InstanceInput {
    @location(1) model_matrix_0: vec3<f32>,
    @location(2) model_matrix_1: vec3<f32>,
    @location(3) model_matrix_2: vec3<f32>,
    @location(4) color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> ortho_matrix: mat4x4<f32>;
@group(1) @binding(0) var<uniform> radius: f32;

@vertex
fn vs_main(in: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat3x3<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
    );
    
    out.clip_position = ortho_matrix *  vec4<f32>(model_matrix * vec3<f32>(in.position, 1.0), 1.0);
    out.clip_position.z = 0.0;

    out.color = instance.color;
    out.rect_position = (model_matrix * vec3<f32>(in.position, 0.0)).xy;

    out.scale = (model_matrix * vec3<f32>(vec2<f32>(1.0), 0.0)).xy;

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color;
    let radius = 25.0;

    if in.rect_position.x < radius && in.rect_position.y < radius {
        let l = length(in.rect_position - vec2<f32>(radius));

        var alpha = 1.0;
        if l > radius {
            alpha = 0.0;
        }
        color = vec4<f32>(color.xyz, alpha);
    } else if in.rect_position.x > in.scale.x - radius && in.rect_position.y < radius {
        let l = length(in.rect_position - vec2<f32>(in.scale.x - radius, radius));

        var alpha = 1.0;
        if l > radius {
            alpha = 0.0;
        }
        color = vec4<f32>(color.xyz, alpha);
    } else if in.rect_position.x > in.scale.x - radius && in.rect_position.y > in.scale.y - radius {
        let l = length(in.rect_position - vec2<f32>(in.scale.x - radius, in.scale.y - radius));

        var alpha = 1.0;
        if l > radius {
            alpha = 0.0;
        }
        color = vec4<f32>(color.xyz, alpha);
    } else if in.rect_position.x < radius && in.rect_position.y > in.scale.y - radius {
        let l = length(in.rect_position - vec2<f32>(radius, in.scale.y - radius));

        var alpha = 1.0;
        if l > radius {
            alpha = 0.0;
        }
        color = vec4<f32>(color.xyz, alpha);
    }

    return vec4<f32>(color);
}