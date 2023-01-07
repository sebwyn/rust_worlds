use crate::{graphics::{RenderApi, RenderPipeline, RenderPipelineDescriptor, AttachmentAccess, Attachment, ShaderDescriptor, RenderPrimitive}, core::Event};

use super::{font::Font, render_structs::{TextureTransform, CharInstance, TexturedVert, Instance, Vert, Transform2d}};

pub struct UiRenderer {
    text_pipeline: RenderPipeline,
    rect_pipeline: RenderPipeline,
    char_instances: Vec<CharInstance>,
    rect_instances: Vec<Instance>,
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

                self.char_instances.push(transform.into());
                x += font_character.advance;
            }
        }
    }

    pub fn put_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: [f32; 4]) {
        let transform = Transform2d { 
            position: [x as f32, y as f32], 
            scale: [width as f32, height as f32], 
            rotation: 0.0 
        };
        self.rect_instances.push(Instance::new(transform, color));
    }
}

impl UiRenderer {
    const INDICES: [u32; 6] = [ 0, 1, 2, 3, 2, 1];
    const TEXTURED_VERTICES: [TexturedVert; 4] = [ 
        TexturedVert { position: [  0f32,  0f32 ], tex_coord: [ 0f32, 1f32 ]},
        TexturedVert { position: [  1f32,  0f32 ], tex_coord: [ 1f32, 1f32 ]},
        TexturedVert { position: [  0f32,  1f32 ], tex_coord: [ 0f32, 0f32 ]},
        TexturedVert { position: [  1f32,  1f32 ], tex_coord: [ 1f32, 0f32 ]},
    ];

    const VERTICES: [Vert; 4] = [ 
        Vert { position: [  0f32,  0f32 ]},
        Vert { position: [  1f32,  0f32 ]},
        Vert { position: [  0f32,  1f32 ]},
        Vert { position: [  1f32,  1f32 ]},
    ];

    pub fn new(render_api: &RenderApi) -> Self {
        let mut text_pipeline = render_api.create_instanced_render_pipeline::<TexturedVert, CharInstance>(RenderPipelineDescriptor { 
            attachment_accesses: vec![AttachmentAccess { clear_color: None, attachment: Attachment::Swapchain }], 
            shader: &ShaderDescriptor { file: "shaders/text.wgsl" }, 
            primitive: RenderPrimitive::Triangles 
        });

        text_pipeline.vertices(&Self::TEXTURED_VERTICES);
        text_pipeline.indices(&Self::INDICES);

        let font_map = Font::new("resources/verdana_sdf.fnt").unwrap();
        let font_texture= render_api.load_texture(&font_map.image_path);

        let texture_binding = text_pipeline.shader().get_texture_binding("font").unwrap();
        text_pipeline.shader().update_texture(&texture_binding, &font_texture, Some(&render_api.create_sampler())).unwrap();

        let ortho_matrix: [[f32; 4]; 4] = cgmath::ortho(0.0, 800.0, 0.0, 600.0, -2.0, 2.0).into();

        let ortho_matrix_binding = text_pipeline.shader().get_uniform_binding("ortho_matrix").unwrap();
        text_pipeline.shader().set_uniform(&ortho_matrix_binding, ortho_matrix).unwrap();


        
        let mut rect_pipeline = render_api.create_instanced_render_pipeline::<Vert, Instance>(RenderPipelineDescriptor { 
            attachment_accesses: vec![AttachmentAccess { clear_color: Some([0f64, 0f64, 0f64, 0f64]), attachment: Attachment::Swapchain }], 
            shader: &ShaderDescriptor { file: "shaders/rect.wgsl" }, 
            primitive: RenderPrimitive::Triangles 
        });

        rect_pipeline.vertices(&Self::VERTICES);
        rect_pipeline.indices(&Self::INDICES);

        let ortho_matrix_binding = rect_pipeline.shader().get_uniform_binding("ortho_matrix").unwrap();
        rect_pipeline.shader().set_uniform(&ortho_matrix_binding, ortho_matrix).unwrap();

        Self {
            text_pipeline,
            rect_pipeline,
            char_instances: Vec::new(),
            rect_instances: Vec::new(),
            font: font_map,
        }
    }

    pub fn update(&mut self, events: &[Event]) {
        let resize = events.iter().rev().find(|r| matches!(r, Event::WindowResized(..)));
        if let Some(Event::WindowResized((width, height))) = resize {
            let ortho_matrix: [[f32; 4]; 4] = cgmath::ortho(0.0, *width as f32, 0.0, *height as f32, -2.0, 2.0).into();

            let ortho_matrix_binding = self.text_pipeline.shader().get_uniform_binding("ortho_matrix").unwrap();
            self.text_pipeline.shader().set_uniform(&ortho_matrix_binding, ortho_matrix).unwrap();

            let ortho_matrix_binding = self.rect_pipeline.shader().get_uniform_binding("ortho_matrix").unwrap();
            self.rect_pipeline.shader().set_uniform(&ortho_matrix_binding, ortho_matrix).unwrap();
        }
    }

    pub fn render(&mut self, surface_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        self.rect_pipeline.instances(&self.rect_instances);
        self.rect_pipeline.render(surface_view, encoder);
        self.rect_instances.clear();

        self.text_pipeline.instances(&self.char_instances);
        self.text_pipeline.render(surface_view, encoder);
        self.char_instances.clear();
    }
}