use std::rc::Rc;

use image::Rgba;

use crate::graphics::{RenderApi, RenderContext};
use crate::graphics::{RenderPipeline, RenderPipelineDescriptor, AttachmentAccess, Attachment, RenderPrimitive};
use crate::graphics::TextureBinding;
use crate::graphics::ShaderDescriptor;
use crate::graphics::Texture;
use crate::Vert;
use crate::graphics::wgsl_types::Vec2;

use rayon::prelude::*;

pub struct Voxels {
    screen_texture: Texture,
    pipeline: RenderPipeline,
    screen_texture_binding: TextureBinding,

    image_buffer: Option<image::ImageBuffer<Rgba<u8>, Vec<u8>>>,

    //need to maintain a context so we can write our textures
    context: Rc<RenderContext>,

}

impl Voxels {
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

        Self {
            screen_texture,
            pipeline,
            screen_texture_binding,
            image_buffer,

            context: api.render_context.clone(),
        }
    }
    
    //some solid parallized cpu raycasting
    pub fn render(&mut self, surface_view: &wgpu::TextureView, width: u32, height: u32) {
        //render our voxels to an image buffer here 
        //create a clear image buffer and set it, clear color for retards
        let image = std::mem::take(&mut self.image_buffer).unwrap();
        let mut rendered_image = image.into_raw();

        rendered_image.par_chunks_mut(4).for_each(|pixel: &mut [u8]| {
            let color: [f32; 4] = [0.8, 0.2, 0.7, 1.0];
            
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
