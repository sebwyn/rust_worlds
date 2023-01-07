use itertools::Itertools;

use crate::{graphics::{RenderApi, RenderPipeline, RenderPipelineDescriptor, AttachmentAccess, Attachment, ShaderDescriptor, RenderPrimitive}, core::Event};

use super::{font::Font, render_structs::{TextureTransform, CharInstance, TexturedVert}};

pub struct UiRenderer {
    pipeline: RenderPipeline,
    instances: Vec<CharInstance>,
    font: Font,
}

impl UiRenderer {
    pub fn put_string(&mut self, text: &str, mut x: u32, y: u32) {
        for c in text.as_bytes().iter() {
            if let Some(font_character) = self.font.get_character(*c as char) {
                let transform = TextureTransform { 
                    position: [(x as i32 + font_character.offset_x) as f32, (y as i32 - font_character.offset_y - font_character.height as i32 - 40) as f32], 
                    scale: [font_character.width as f32, font_character.height as f32], 
                    rotation: 0.0,

                    tex_position: [font_character.tex_x, font_character.tex_y],
                    tex_scale: [font_character.tex_width, font_character.tex_height]
                };

                self.instances.push(CharInstance::from(transform.clone()));
                x += font_character.advance;
            }
        }
    }
}

impl UiRenderer {
    const INDICES: [u32; 6] = [ 0, 1, 2, 3, 2, 1];
    const VERTICES: [TexturedVert; 4] = [ 
        TexturedVert { position: [  0f32,  0f32 ], tex_coord: [ 0f32, 1f32 ]},
        TexturedVert { position: [  1f32,  0f32 ], tex_coord: [ 1f32, 1f32 ]},
        TexturedVert { position: [  0f32,  1f32 ], tex_coord: [ 0f32, 0f32 ]},
        TexturedVert { position: [  1f32,  1f32 ], tex_coord: [ 1f32, 0f32 ]},
    ];

    pub fn new(render_api: &RenderApi) -> Self {
        let mut pipeline = render_api.create_instanced_render_pipeline::<TexturedVert, CharInstance>(RenderPipelineDescriptor { 
            attachment_accesses: vec![AttachmentAccess { clear_color: Some([0f64, 0f64, 0f64, 0f64]), attachment: Attachment::Swapchain }], 
            shader: &ShaderDescriptor { file: "shaders/text.wgsl" }, 
            primitive: RenderPrimitive::Triangles 
        });

        pipeline.vertices(&Self::VERTICES);
        pipeline.indices(&Self::INDICES);

        let font_map = Font::new("resources/verdana_sdf.fnt").unwrap();
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

    pub fn update(&mut self, events: &[Event]) {
        let resize = events.iter().rev().find(|r| matches!(r, Event::WindowResized(..)));
        if let Some(Event::WindowResized((width, height))) = resize {
            let ortho_matrix: [[f32; 4]; 4] = cgmath::ortho(0.0, *width as f32, 0.0, *height as f32, -2.0, 2.0).into();

            let ortho_matrix_binding = self.pipeline.shader().get_uniform_binding("ortho_matrix").unwrap();
            self.pipeline.shader().set_uniform(&ortho_matrix_binding, ortho_matrix).unwrap();
        }
    }

    pub fn render(&mut self, surface_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        self.pipeline.instances(&self.instances);
        self.pipeline.render(surface_view, encoder);
        self.instances.clear();
    }
}