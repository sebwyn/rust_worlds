use super::Shader;
use super::{Attachment, AttachmentAccess, RenderApi, RenderContext};

use std::rc::Rc;

use wgpu::util::DeviceExt;

#[derive(Clone)]
pub enum RenderPrimitive {
    Triangles,
    Lines
}

impl RenderPrimitive {
    fn num_vertices(&self) -> u32 {
        match self {
            RenderPrimitive::Triangles => 3,
            RenderPrimitive::Lines => 2,
        }
    }
}

impl Into<wgpu::PrimitiveTopology> for RenderPrimitive {
    fn into(self) -> wgpu::PrimitiveTopology {
        match self {
            RenderPrimitive::Triangles => wgpu::PrimitiveTopology::TriangleList,
            RenderPrimitive::Lines => wgpu::PrimitiveTopology::LineList,
        }
    }
}

pub trait Vertex : bytemuck::Pod {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub struct RenderPipelineDescriptor {
    pub attachment_accesses: Vec<AttachmentAccess>,
    pub shader: Rc<Shader>,
    pub primitive: RenderPrimitive
}

pub struct Vertices {
    vertices: Option<(u32, wgpu::Buffer)>,
    indices: Option<(u32, wgpu::Buffer)>,
}

pub struct RenderPipeline {
    shader: Rc<Shader>,

    attachment_accesses: Vec<AttachmentAccess>,
    _layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,

    has_vertex: bool,
    vertices: Option<Vertices>,

    context: Rc<RenderContext>
}

//temporary
impl RenderPipeline {
    pub fn new(descriptor: RenderPipelineDescriptor, api: &RenderApi) -> Self {
        Self::create(descriptor, &[], api)
    }
    
    pub fn new_with_vertex<T>(descriptor: RenderPipelineDescriptor, api: &RenderApi) -> Self 
    where
        T: Vertex
    {
        let mut instance = Self::create(descriptor, &[T::desc()], api);
        instance.has_vertex = true;
        instance
    }

    pub fn set_vertices<T>(&mut self, new_vertices: &[T]) 
    where
        T: Vertex
    {
        assert!(self.has_vertex, "Setting vertex buffer on a pipeline with no vertices!");

        //self.context.queue().write_buffer(&vertices.vertex_buffer, 0, bytemuck::cast_slice(new_vertices));
        let new_buffer = self.context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(new_vertices), 
            usage: wgpu::BufferUsages::VERTEX
        });
        let new_vertex_buffer = Some((new_vertices.len() as u32, new_buffer));

        if let Some(vertices) = &mut self.vertices {
            vertices.vertices = new_vertex_buffer;
        } else {
            self.vertices = Some(Vertices { vertices: new_vertex_buffer, indices: None });
        }
    }

    pub fn set_indices(&mut self, new_indices: &[u32]) {
        assert!(self.has_vertex, "Setting an index buffer on a pipeline with no vertices!");
        
        //self.context.queue().write_buffer(&vertices.vertex_buffer, 0, bytemuck::cast_slice(new_vertices));
        let new_buffer = self.context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(new_indices), 
            usage: wgpu::BufferUsages::VERTEX
        });
        let new_index_buffer = Some((new_indices.len() as u32, new_buffer));

        if let Some(vertices) = &mut self.vertices {
            vertices.indices = new_index_buffer;
        } else {
            self.vertices = Some(Vertices { vertices: None, indices: new_index_buffer });
        }
    }

    fn generate_framebuffer<'a>(&'a self, surface_view: &'a wgpu::TextureView) -> Vec<Option<wgpu::RenderPassColorAttachment<'a>>> {
        self.attachment_accesses.iter().map(|access| {
            let load_op = match access.clear_color {
                Some(rgba) => wgpu::LoadOp::Clear(wgpu::Color { r: rgba[0], g: rgba[1], b: rgba[2], a: rgba[3]}),
                None => wgpu::LoadOp::Load,
            };

            if let Attachment::Texture(texture) = &access.attachment {
                Some(wgpu::RenderPassColorAttachment { 
                    view: texture.view(), 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: load_op,
                        store: true,
                    }
                })
            } else { 
                Some(wgpu::RenderPassColorAttachment { 
                    view: surface_view, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: load_op,
                        store: true,
                    }
                })
            }

        }).collect()
    }

    //this will be shitty at first, because we're generating a command buffer for every pipeline
    pub fn render(&self, surface_view: &wgpu::TextureView) {

        let verts = if let Some(vertices) = &self.vertices {
            let (count, vertex_buffer) = vertices.vertices.as_ref().expect("Rendering without any vertices");
            Some((count, vertex_buffer, vertices.indices.as_ref()))
        } else {
            None
        };

        let color_attachments = self.generate_framebuffer(surface_view);

        let mut encoder = self.context.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { 
            label: Some("Triangle Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Triangle Pass"), 
                color_attachments: &color_attachments, 
                depth_stencil_attachment: None
            });

            render_pass.set_pipeline(&self.pipeline);
            self.shader.bind_uniforms(&mut render_pass);

            if let Some((vertex_count, vertex_buffer, indices)) = verts {
                render_pass.set_vertex_buffer(*vertex_count, vertex_buffer.slice(..));
                if let Some((index_count, index_buffer)) = indices {
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..*index_count, 0, 0..1);
                } else {
                    render_pass.draw(0..*vertex_count, 0..1);
                }
            } else {
                render_pass.draw(0..3, 0..1);
            }
        }

        self.context.queue().submit(std::iter::once(encoder.finish()));
    }
}

impl RenderPipeline {

    pub fn create(descriptor: RenderPipelineDescriptor, vertex_descriptions: &[wgpu::VertexBufferLayout], api: &RenderApi) -> Self
    {
        let shader = descriptor.shader; 

        let uniform_layouts = shader.layouts();

        let targets: Vec<Option<wgpu::ColorTargetState>> = 
            descriptor.attachment_accesses.iter()
            .map(|access| {
                if let Attachment::Texture(texture) = &access.attachment {
                    Some(wgpu::ColorTargetState {
                        format: *texture.format(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                } else { 
                    Some(wgpu::ColorTargetState {
                        format: api.surface().config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                }
            }).collect();


        let layout =
            api.context().device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &uniform_layouts,
                push_constant_ranges: &[],
            });

        //create our pipeline here
        let pipeline =
            api.context().device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader.module(),
                    entry_point: shader.vs_entry_point(),
                    buffers: vertex_descriptions,
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader.module(),
                    entry_point: shader.fs_entry_point(),
                    targets: &targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: descriptor.primitive.clone().into(),
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,                  //Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Self {
            shader,
            attachment_accesses: descriptor.attachment_accesses,
            _layout: layout,
            pipeline,
            
            context: api.render_context.clone(),

            has_vertex: false,
            vertices: None
        }
    }

}
