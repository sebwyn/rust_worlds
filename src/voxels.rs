use std::rc::Rc;

use image::Rgba;

use crate::graphics::{RenderApi, RenderContext};
use crate::graphics::{RenderPipeline, RenderPipelineDescriptor, AttachmentAccess, Attachment, RenderPrimitive};
use crate::graphics::TextureBinding;
use crate::graphics::ShaderDescriptor;
use crate::graphics::Texture;
use crate::Vert;
use crate::graphics::wgsl_types::{Vec2, Vec3};

use rayon::prelude::*;

#[derive(Copy, Clone, Debug)]
struct Voxel {
    exists: bool
}

pub struct Voxels {
    screen_texture: Texture,
    pipeline: RenderPipeline,
    screen_texture_binding: TextureBinding,

    image_buffer: Option<image::ImageBuffer<Rgba<u8>, Vec<u8>>>,

    dimensions: (u32, u32, u32),
    voxels: Vec<Voxel>,

    //need to maintain a context so we can write our textures
    context: Rc<RenderContext>,

}

impl Voxels {
    fn voxel_pos_from_index(i: u32, dimensions: (u32, u32, u32)) -> (u32, u32, u32) {
        let v_x = i % dimensions.0;
        let v_y = (i / dimensions.0) % dimensions.1;
        let v_z = i / (dimensions.0 * dimensions.1);

        (v_x, v_y, v_z)
    }

    const INDICES: [u32; 6] = [ 0, 1, 2, 3, 2, 1];
    const VERTICES: [Vert; 4] = [ 
        Vert { position: Vec2 { x: -1f32, y: -1f32 }, tex_coord: Vec2 { x: 0f32, y: 0f32}},
        Vert { position: Vec2 { x:  1f32, y: -1f32 }, tex_coord: Vec2 { x: 1f32, y: 0f32}},
        Vert { position: Vec2 { x: -1f32, y:  1f32 }, tex_coord: Vec2 { x: 0f32, y: 1f32}},
        Vert { position: Vec2 { x:  1f32, y:  1f32 }, tex_coord: Vec2 { x: 1f32, y: 1f32}},
    ];

    //for now default to using Bgra
    pub fn new(api: &RenderApi) -> Self {
        let width = api.surface().config.width;
        let height = api.surface().config.height;

        let screen_texture = api.create_image_texture::<Rgba<u8>>(width, height);

        let mut pipeline = api.create_render_pipeline_with_vertex::<Vert>(RenderPipelineDescriptor { 
            attachment_accesses: vec![
                AttachmentAccess {
                    clear_color: Some([0f64; 4]),
                    attachment: Attachment::Swapchain,
                }
            ], 
            shader: &ShaderDescriptor {
                file: "voxels.wgsl",
            }, 
            primitive: RenderPrimitive::Triangles,
        });

        let screen_texture_binding = pipeline.get_texture_binding("screen_texture").expect("Cannot find screen texture");

        pipeline.set_vertices(&Self::VERTICES);
        pipeline.set_indices(&Self::INDICES);

        let image_buffer = Some(image::ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height));

        //initialize our voxel data structure here too

        let dimensions = (100, 100, 100);
        let sphere_center = Vec3 { x: 50f32, y: 50f32, z: 50f32 };

        //todo, do a flat map
        //do it as an 100 by 100 by 100 flat array
        let voxels: Vec<Voxel> = (0..dimensions.0*dimensions.1*dimensions.2).into_par_iter().map(|i| {
            //generate a vec3 center from our index
            let (v_x, v_y, v_z) = Self::voxel_pos_from_index(i, dimensions);

            //calculate a vec3 which is the center of this voxel
            let voxel_center = Vec3 { x: v_x as f32 + 0.5, y: v_y as f32 + 0.5, z: v_z as f32 + 0.5 };
            //calculate the distance to the center of our sphere
            let distance = (sphere_center - voxel_center).magnitude();

            let exists = if distance < 25f32 {
                true
            } else {
                false
            };

            Voxel {
                exists
            }
        }).collect();

        Self {
            screen_texture,
            pipeline,
            screen_texture_binding,
            image_buffer,

            dimensions,
            voxels,

            context: api.render_context.clone(),
        }
    }
    
    //some solid parallized cpu raycasting
    pub fn render(&mut self, surface_view: &wgpu::TextureView, width: u32, height: u32) {
        //render our voxels to an image buffer here 
        //create a clear image buffer and set it, clear color for retards
        let image = std::mem::take(&mut self.image_buffer).unwrap();
        let mut rendered_image = image.into_raw();

        let camera_pos = Vec3 { x: 50f32, y: 50.01f32, z: 0f32 };
        let near = 2.0f32; //this will change the zoom????

        let voxel_from_pos = |x: i32, y: i32, z: i32| -> Voxel {
            //do the inverse of voxel pos from index to get an index from a position
            if 0 < x && (x as u32) < self.dimensions.0 && 0 < y && (y as u32) < self.dimensions.1 && 0 < z && (z as u32) < self.dimensions.2 {
                let x = x as u32;
                let y = y as u32;
                let z = z as u32;

                let index = x + y * self.dimensions.0 + z * (self.dimensions.0 * self.dimensions.1);
                self.voxels.get(index as usize).map(|v| *v).unwrap_or(Voxel { exists: false })
            } else {
                Voxel { exists: false }
            }
        };

        println!("voxel: {:?}", voxel_from_pos(50, 50, 0));

        //when rendering positions start in the top left
        rendered_image.par_chunks_exact_mut(4).enumerate().for_each(|(i, pixel)| {
            let mut color: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
            //calculate our screen pos from the index

            //start by transforming out index into a vec2
            let mut p_x = (i as u32 % width) as f32;
            let mut p_y = (i as u32 / width) as f32;
            //transform position to have an origin at the center
            p_x -= width as f32 / 2f32;
            p_y -= height as f32 / 2f32;


            //figure out our ray from the postion and camera position, for now assuming the camera points towards positive z (near is positive)
            //we divide by zoom?? here
            let origin = camera_pos;
            let world_ray = (Vec3 { x: p_x / 100f32, y: p_y / 100f32, z:  near }).normalize();
            
            //should vectorize all these operations, no reason to be this verbose

            let step_x = world_ray.x.signum() as i32;
            let step_y = world_ray.y.signum() as i32;
            let step_z = world_ray.z.signum() as i32;

            let t_delta_x = 1.0 / world_ray.x;
            let t_delta_y = 1.0 / world_ray.y;
            let t_delta_z = 1.0 / world_ray.z;
            
            //get the fractional component of the position to see where we lie in the grid
            let world_x_fract = origin.x.abs() % 1.0;
            let world_y_fract = origin.y.abs() % 1.0;
            let world_z_fract = origin.z.abs() % 1.0;
            let mut t_max_x = t_delta_x * if world_ray.x > 0f32 { 1f32 - world_x_fract } else { world_x_fract };
            let mut t_max_y = t_delta_y * if world_ray.y > 0f32 { 1f32 - world_y_fract } else { world_y_fract };
            let mut t_max_z = t_delta_z * if world_ray.z > 0f32 { 1f32 - world_z_fract } else { world_z_fract };
            
            //initial voxel, start at 0, 0 here
            let mut voxel_x = origin.x.floor() as i32;
            let mut voxel_y = origin.y.floor() as i32;
            let mut voxel_z = origin.z.floor() as i32;

            //we now have a starting voxel we should check for collision???
            
            //cap the number of iterations at zero
            for i in 0..100 {
                if t_max_x.abs() < t_max_y.abs() && t_max_x.abs() < t_max_z.abs() {
                    t_max_x= t_max_x + t_delta_x;
                    voxel_x += step_x;
                } else if t_max_y.abs() < t_max_z.abs() {
                    t_max_y= t_max_y + t_delta_y;
                    voxel_y += step_y;
                } else {
                    t_max_z = t_max_z + t_delta_z;
                    voxel_z += step_z;
                }

                //we have now found another voxel, maybe colorize this one somehow????
                if voxel_from_pos(voxel_x, voxel_y, voxel_z).exists {
                    color = [1f32, 1f32, 1f32, 1f32];
                    break;
                };
            }

            //set the color here based on the voxel that we ended at
            if color[0] > 0.5 {  
                color = [ voxel_x as f32 / 100f32, voxel_y as f32 / 100f32, voxel_z as f32 / 100f32, 1.0 ];
            }

            pixel.clone_from_slice(&color.into_iter().map(|c| (c * 255f32) as u8).collect::<Vec<u8>>());
        });

        let image_buffer: image::ImageBuffer::<Rgba<u8>, Vec<u8>> = 
            image::ImageBuffer::from_raw(width, height, rendered_image).expect("Couldn't reconstruct image");
        
        self.screen_texture.write_image(&image_buffer, self.context.queue());
        self.pipeline.update_texture(&self.screen_texture_binding, &self.screen_texture).expect("Failed to update screen texture!");
        self.image_buffer.replace(image_buffer);

        self.pipeline.render(surface_view);
    }
}
