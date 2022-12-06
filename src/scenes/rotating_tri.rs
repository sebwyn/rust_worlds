use std::time::Instant;

use crate::graphics::RenderApi;
use crate::graphics::Sampler;
use crate::graphics::ShaderDescriptor;
use crate::graphics::{RenderPipelineDescriptor, RenderPrimitive, RenderPipeline};
use crate::graphics::{Attachment, AttachmentAccess};
use crate::Vert;
use crate::graphics::wgsl_types::Vec2;

fn pos_to_tex(pos: Vec2) -> Vec2 {
    Vec2 { x: (pos.x + 1f32) / 2f32, y: (pos.y + 1f32) / 2f32 }
}

fn normalize_position(pos: Vec2, width: u32, height: u32) -> Vec2 {
    Vec2 { x: pos.x / (width as f32/ 2f32), y: pos.y / (height as f32 / 2f32)}
}

pub struct RotatingTri {
    pipeline: RenderPipeline,

    positions: Vec<Vec2>,
    start_time: Instant,
}

impl RotatingTri  {
    pub fn new(api: &RenderApi) -> Self {
        let mut pipeline = api.create_render_pipeline_with_vertex::<Vert>(RenderPipelineDescriptor { 
            attachment_accesses: vec![
                AttachmentAccess { 
                    clear_color: Some([0f64, 0f64, 0f64, 1f64]), 
                    attachment: Attachment::Swapchain 
                }
            ], 
            shader: &ShaderDescriptor {
                file: "basic.wgsl" 
            }, 
            primitive: RenderPrimitive::Triangles 
        });

        let texture = api.load_texture("tex.jpeg");
        let texture_binding = pipeline.shader().get_texture_binding("diffuse").expect("Can't get texture uniform");
        let sampler = Some(Sampler::new(api.context().device()));
        pipeline.shader().update_texture(&texture_binding, &texture, sampler.as_ref()).expect("failed to set texture");

        let magnitude = 120f32;
        let positions: Vec<Vec2> = (0..3).map(|index| index as f32 * (2f32*std::f32::consts::PI) / 3f32).map(|angle| Vec2 { x: magnitude * f32::cos(angle), y: magnitude * f32::sin(angle)}).collect();

        let start_time = Instant::now();

        Self {
            pipeline,
            positions,
            start_time,

        }
    }
    
    pub fn render(&mut self, surface_texture: &wgpu::TextureView, width: u32, height: u32) {
        let angle = 2f32 * std::f32::consts::PI / 200f32;
        let theta = (self.start_time.elapsed().as_millis() % 5000u128) as f32 / 2500f32 * 2f32 * std::f32::consts::PI;
        
        let positions: Vec<Vec2> = self.positions.iter().map(|position| {
            position.rotate(angle)
        }).collect();

        let radius = 60f32;
        let vertices: Vec<Vert> = self.positions.iter().map(|position| {
            let position = 
                *position + Vec2 { x:  radius * -f32::cos(theta), y: radius*f32::sin(theta) };
            let position = normalize_position(position, width, height);
            Vert { position, tex_coord: pos_to_tex(position) }
        }).collect();
        self.pipeline.set_vertices(&vertices);

        self.positions = positions;

        self.pipeline.render(surface_texture);

    }
}
