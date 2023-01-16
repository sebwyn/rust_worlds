use crate::{graphics::{RenderApi, RenderPipeline, RenderPipelineDescriptor, AttachmentAccess, Attachment, ShaderDescriptor, RenderPrimitive, Texture, Sampler}, core::Event};

use super::{font::Font, render_structs::{TextureTransform, TexturedInstance, TexturedVert, Instance, Vert, Transform2d}};

//Todo use depth buffering to make this cleaner, with optional depths and such
//for now rects -> sprites -> text
//for now for simplicity only one sprite map is supported
pub struct UiRenderer {
    text_pipeline: RenderPipeline,
    rect_pipeline: RenderPipeline,
    sprite_pipeline: RenderPipeline,
    char_instances: Vec<TexturedInstance>,
    rect_instances: Vec<Instance>,
    sprite_instances: Vec<TexturedInstance>,
    font: Font,
}

pub struct Layout {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Layout {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height
        }
    }
}

impl UiRenderer {
    pub fn put_image(&mut self, layout: &Layout, texture_layout: &Layout) {
        let transform = TextureTransform {
            position: [layout.x, layout.y - layout.height],
            scale: [layout.width, layout.height],
            rotation: 0.0,

            tex_position: [texture_layout.x, texture_layout.y],
            tex_scale: [texture_layout.width, texture_layout.height],
        };

        self.sprite_instances.push(transform.into());
    }

    pub fn set_sprite_map(&mut self, texture: &Texture, sampler: Option<&Sampler>) {
        self.sprite_pipeline.shader().update_texture("sprites", texture, sampler).unwrap();
    }

    //positioned using the upper left corner
    pub fn put_text_box(&mut self, text: &str, color: [f32; 4], layout: &Layout) {
        //apply a padding of 10 pixels
        let text_layout = 
            Layout {
                x: layout.x + 10.0,
                y: layout.y - 10.0,
                width: layout.width - 20.0,
                height: layout.width - 20.0,
            };

        self.put_rect(color, layout);
        self.put_text(text, &text_layout);
    }

    //text is position using the top left corner
    pub fn put_text(&mut self, text: &str, layout: &Layout) {
        let mut curr_x = layout.x as f32;
        let mut curr_y = layout.y as f32;
        
        let line_height = 21.0;
        for c in text.as_bytes().iter() {
            let c = *c as char;
            if let Some(font_character) = self.font.get_character(c) {
                let transform = TextureTransform { 
                    position: [
                        (curr_x + font_character.offset_x as f32), 
                        (curr_y - (font_character.offset_y as f32 + font_character.height as f32))
                    ], 
                    scale: [font_character.width as f32, font_character.height as f32], 
                    rotation: 0.0,

                    tex_position: [font_character.tex_x, font_character.tex_y],
                    tex_scale: [font_character.tex_width, font_character.tex_height]
                };

                self.char_instances.push(transform.into());
                curr_x += font_character.advance as f32;
                if curr_x > layout.width as f32 {
                    curr_y -= line_height;
                    curr_x = layout.x as f32;
                }
            }

            if c == '\n' {
                curr_y -= line_height;
            }
        }
    }

    //x and y are the upper left corner of the text
    pub fn _put_string(&mut self, text: &str, x: u32, y: u32, font_size: u32) {
        let mut x = x as f32;
        let y = y as f32;
        
        let font_scale = font_size as f32;
        //let line_height = 10.0;
        for c in text.as_bytes().iter() {
            if let Some(font_character) = self.font.get_character(*c as char) {
                let transform = TextureTransform { 
                    position: [
                        (x + font_character.offset_x as f32 * font_scale), 
                        (y - (font_character.offset_y as f32 - font_character.height as f32) * font_scale)
                    ], 
                    scale: [font_character.width as f32 * font_scale, font_character.height as f32 * font_scale], 
                    rotation: 0.0,

                    tex_position: [font_character.tex_x, font_character.tex_y],
                    tex_scale: [font_character.tex_width, font_character.tex_height]
                };

                self.char_instances.push(transform.into());
                x += font_character.advance as f32 * font_scale;
            }
        }
    }

    //x and y are the upper left corner of the box
    pub fn put_rect(&mut self, color: [f32; 4], layout: &Layout) {
        let transform = Transform2d { 
            position: [layout.x as f32, (layout.y - layout.height) as f32], 
            scale: [layout.width as f32, layout.height as f32], 
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
        let ortho_matrix: [[f32; 4]; 4] = cgmath::ortho(0.0, 800.0, 0.0, 600.0, -2.0, 2.0).into();
        
        let mut text_pipeline = render_api.create_instanced_render_pipeline::<TexturedVert, TexturedInstance>(RenderPipelineDescriptor { 
            attachment_accesses: vec![AttachmentAccess { clear_color: None, attachment: Attachment::Swapchain }], 
            shader: &ShaderDescriptor { file: "shaders/text.wgsl" }, 
            primitive: RenderPrimitive::Triangles 
        });

        text_pipeline.vertices(&Self::TEXTURED_VERTICES);
        text_pipeline.indices(&Self::INDICES);

        let font_map = Font::new("resources/verdana.fnt").unwrap();
        let font_texture= render_api.load_texture(&font_map.image_path);

        text_pipeline.shader().update_texture("font", &font_texture, Some(&render_api.create_sampler())).unwrap();
        text_pipeline.shader().set_uniform("ortho_matrix", ortho_matrix).unwrap();


        let mut rect_pipeline = render_api.create_instanced_render_pipeline::<Vert, Instance>(RenderPipelineDescriptor { 
            attachment_accesses: vec![AttachmentAccess { clear_color: Some([0f64, 0f64, 0f64, 0f64]), attachment: Attachment::Swapchain }], 
            shader: &ShaderDescriptor { file: "shaders/rect.wgsl" }, 
            primitive: RenderPrimitive::Triangles 
        });

        rect_pipeline.vertices(&Self::VERTICES);
        rect_pipeline.indices(&Self::INDICES);

        rect_pipeline.shader().set_uniform("ortho_matrix", ortho_matrix).unwrap();
        rect_pipeline.shader().set_uniform("radius", 25f32).unwrap();
        
        let mut sprite_pipeline = render_api.create_instanced_render_pipeline::<Vert, Instance>(RenderPipelineDescriptor { 
            attachment_accesses: vec![AttachmentAccess { clear_color: Some([0f64, 0f64, 0f64, 0f64]), attachment: Attachment::Swapchain }], 
            shader: &ShaderDescriptor { file: "shaders/sprite.wgsl" }, 
            primitive: RenderPrimitive::Triangles 
        });

        sprite_pipeline.vertices(&Self::VERTICES);
        sprite_pipeline.indices(&Self::INDICES);

        sprite_pipeline.shader().set_uniform("ortho_matrix", ortho_matrix).unwrap();

        Self {
            text_pipeline,
            rect_pipeline,
            sprite_pipeline,
            char_instances: Vec::new(),
            rect_instances: Vec::new(),
            sprite_instances: Vec::new(),
            font: font_map,
        }
    }

    pub fn update(&mut self, events: &[Event]) {
        let resize = events.iter().rev().find(|r| matches!(r, Event::WindowResized(..)));
        if let Some(Event::WindowResized((width, height))) = resize {
            let ortho_matrix: [[f32; 4]; 4] = cgmath::ortho(0.0, *width as f32, 0.0, *height as f32, -2.0, 2.0).into();

            self.text_pipeline.shader().set_uniform("ortho_matrix", ortho_matrix).unwrap();
            self.rect_pipeline.shader().set_uniform("ortho_matrix", ortho_matrix).unwrap();
        }
    }

    pub fn render(&mut self, surface_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        self.rect_pipeline.instances(&self.rect_instances);
        self.rect_pipeline.render(surface_view, encoder);
        self.rect_instances.clear();

        self.sprite_pipeline.instances(&self.sprite_instances);
        self.sprite_pipeline.render(surface_view, encoder);
        self.sprite_instances.clear();

        self.text_pipeline.instances(&self.char_instances);
        self.text_pipeline.render(surface_view, encoder);
        self.char_instances.clear();
    }
}