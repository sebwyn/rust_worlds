use std::mem;

use crate::graphics::{RenderApi, RenderPipeline, RenderPipelineDescriptor, AttachmentAccess, Attachment, ShaderDescriptor, RenderPrimitive, Vertex};

use super::font::Font;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vert { 
    pub position: [f32; 2] 
}

impl Vert {
    const ATTRIBS: [wgpu::VertexAttribute; 1] =
        wgpu::vertex_attr_array![0 => Float32x2];
}

impl Vertex for Vert {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Clone)]
struct Transform2d {
    position: [f32; 2],
    scale: [f32; 2],
    rotation: f32,

    tex_position: [f32; 2],
    tex_scale: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Instance {
    model: [[f32; 3]; 3],
    tex_coords: [[f32; 3]; 3],
}

impl From<Transform2d> for Instance {
    fn from(transform: Transform2d) -> Self {
        let rotation_matrix = cgmath::Matrix3::from_angle_z(cgmath::Rad(transform.rotation));
        let scale_matrix = cgmath::Matrix3::from_nonuniform_scale(transform.scale[0], transform.scale[1]);

        let tex_scale_matrix = cgmath::Matrix3::from_nonuniform_scale(transform.tex_scale[0], transform.tex_scale[1]);
        Self {
            model: (cgmath::Matrix3::from_translation(transform.position.into()) * rotation_matrix * scale_matrix).into(),
            tex_coords: (cgmath::Matrix3::from_translation(transform.tex_position.into()) * tex_scale_matrix).into()
        }
    }
}

impl Vertex for Instance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },

                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 15]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct UiRenderer {
    pipeline: RenderPipeline,
    instances: Vec<Instance>,
    font: Font,
}

impl UiRenderer {
    pub fn push_char(&mut self, c: char, x: u32, y: u32) {
        if let Some(font_character) = self.font.get_character(c) {
            let transform = Transform2d { 
                position: [x as f32, y as f32], 
                scale: [font_character.width as f32, font_character.height as f32], 
                rotation: 0.0,

                tex_position: [font_character.tex_x, font_character.tex_y],
                tex_scale: [font_character.tex_width, font_character.tex_height]
            };

            self.instances.push(Instance::from(transform.clone()));
        }
    }
}

impl UiRenderer {
    const INDICES: [u32; 6] = [ 0, 1, 2, 3, 2, 1];
    const VERTICES: [Vert; 4] = [ 
        Vert { position: [  0f32,  0f32 ]},
        Vert { position: [  1f32,  0f32 ]},
        Vert { position: [  0f32,  1f32 ]},
        Vert { position: [  1f32,  1f32 ]},
    ];

    pub fn new(render_api: &RenderApi) -> Self {
        let mut pipeline = render_api.create_instanced_render_pipeline::<Vert, Instance>(RenderPipelineDescriptor { 
            attachment_accesses: vec![AttachmentAccess { clear_color: Some([0f64, 0f64, 0f64, 0f64]), attachment: Attachment::Swapchain }], 
            shader: &ShaderDescriptor { file: "shaders/text.wgsl" }, 
            primitive: RenderPrimitive::Triangles 
        });

        pipeline.vertices(&Self::VERTICES);
        pipeline.indices(&Self::INDICES);

        let font_map = Font::new("verdana_sdf.fnt").unwrap();
        let font_texture= render_api.load_texture(&font_map.image_path);

        let texture_binding = pipeline.shader().get_texture_binding("font").unwrap();
        pipeline.shader().update_texture(&texture_binding, &font_texture, Some(&render_api.create_sampler())).unwrap();

        let ortho_matrix: [[f32; 4]; 4] = cgmath::ortho(0.0, 800.0, 0.0, 600.0, -2.0, 2.0).into();

        let ortho_matrix_binding = pipeline.shader().get_uniform_binding("ortho_matrix").unwrap();
        pipeline.shader().set_uniform(&ortho_matrix_binding, ortho_matrix).unwrap();

        Self {
            pipeline,
            instances: Vec::new(),
            font: font_map,
        }
    }

    pub fn resize() {}

    pub fn render(&mut self, surface_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        self.pipeline.instances(&self.instances);
        self.pipeline.render(surface_view, encoder);
        self.instances.clear();
    }
}