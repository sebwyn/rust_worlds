struct VertexInput {
    @location(0)  position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    return out;
}

// Fragment shader
@group(0) @binding(0) var voxel_data: texture_2d<u32>;
@group(2) @binding(0) var palette: texture_1d<f32>;

@group(1) @binding(0) var<uniform> resolution: vec2<f32>;
@group(1) @binding(1) var<uniform> camera_position: vec3<f32>;
@group(1) @binding(2) var<uniform> view_matrix: mat4x4<f32>;
@group(1) @binding(3) var<uniform> near: f32;

fn voxel_from_pos(position: vec3<i32>) -> vec4<f32> {
    if(
        0 <= position.x && position.x < 4 &&
        0 <= position.y && position.y < 4 &&
        0 <= position.z && position.z < 4
    ){
        //calculate our tex coord from our y, and z, x coords are contained in one pixel that is 32 bit

        //return true;
        //look up the voxel in our texture
        let row = textureLoad(voxel_data, position.yz, 0).r;
        let color_index = i32(row & (u32(0xFF) << u32(position.x * 8)));

        //return vec4<f32>(f32(position.x) / 4.0, f32(position.y) / 4.0, f32(position.z) / 4.0, 1.0);
        //if color_index > 0 {
            return textureLoad(palette, color_index, 0);
        //}

    }

    return vec4<f32>(-1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //start by transforming out index into a vec2
    //transform position to have an origin at the center
    //var p = (in.position.xy - resolution / 2.0) / 100.0;
    var p = (in.position.xy - resolution / 2.0) / (resolution / 2.0);
    p.y = -1.0 * p.y;
    let screen_world = vec3<f32>(p, -near);
    let mag = length(vec3<f32>(p, near));

    //figure out our ray from the postion and camera position, for now assuming the camera points towards positive z (near is positive)
    //we divide by zoom?? here
    let origin = camera_position;
    var world_ray = (view_matrix * vec4<f32>(normalize(screen_world), 0.0)).xyz;

    //should vectorize all these operations, no reason to be this verbose
    let step = vec3<i32>(sign(world_ray));
    let t_delta = 1.0 / world_ray;

    let world_fract = abs(fract(origin));

    var t_max_x = 0.0;
    if(world_ray.x > 0.0){
        t_max_x = 1.0 - world_fract.x;
    } else {
        t_max_x = world_fract.x;
    };

    var t_max_y = 0.0;
    if(world_ray.y > 0.0){
        t_max_y = 1.0 - world_fract.y;
    } else {
        t_max_y = world_fract.y;
    };

    var t_max_z = 0.0;
    if(world_ray.z > 0.0){
        t_max_z = 1.0 - world_fract.z;
    } else {
        t_max_z = world_fract.z;
    };

    var t_max = t_delta * vec3<f32>(t_max_x, t_max_y, t_max_z);

    var voxel = vec3<i32>(floor(origin));

    //we now have a starting voxel we should check for collision???

    //cap the number of iterations at zero
    var iterations = 0.0;
    for(var i: i32; i < 50; i++) {
        iterations += 0.02;
        if abs(t_max.x) < abs(t_max.y) && abs(t_max.x) < abs(t_max.z) {
            t_max.x = t_max.x + t_delta.x;
            voxel.x += step.x;
        } else if abs(t_max.y) < abs(t_max.z) {
            t_max.y= t_max.y + t_delta.y;
            voxel.y += step.y;
        } else {
            t_max.z = t_max.z + t_delta.z;
            voxel.z += step.z;
        }

        let color = voxel_from_pos(voxel);
        if color.x > -0.1 {
            return color;
            //return vec4<f32>(color.xyz, 1.0);
        }
    }

    return vec4<f32>(0.0);
}
