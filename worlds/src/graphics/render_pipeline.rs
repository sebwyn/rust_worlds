use super::{Shader, ShaderDescriptor};
use super::{RenderApi, RenderContext};
use super::Texture;

use std::rc::Rc;

use wgpu::util::DeviceExt;

//an attachment is a render target
pub enum Attachment {
    Swapchain,
    Texture(Rc<Texture>),
}

pub struct AttachmentAccess {
    pub clear_color: Option<([f64; 4])>,
    pub attachment: Attachment,
}

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

//think about simplifying this, so you only have to specify the vertex attribs
pub trait Vertex : bytemuck::Pod {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub struct RenderPipelineDescriptor<'a> {
    pub attachment_accesses: Vec<AttachmentAccess>,
    pub shader: &'a ShaderDescriptor<'a>,
    pub primitive: RenderPrimitive
}

pub struct RenderPipeline {
    shader: Shader,

    attachment_accesses: Vec<AttachmentAccess>,
    _layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,

    vertices: Option<(u32, wgpu::Buffer)>,
    indices: Option<(u32, wgpu::Buffer)>,

    has_vertex: bool,

    context: Rc<RenderContext>
}

impl RenderPipeline {
    pub fn shader(&mut self) -> &mut Shader { &mut self.shader } 
}

//temporary
impl RenderPipeline {
    pub fn new_with_vertex<T>(descriptor: RenderPipelineDescriptor, api: &RenderApi) -> Self 
    where
        T: Vertex
    {
        Self::create(descriptor, &[T::desc()], api)
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
        self.vertices = Some((new_vertices.len() as u32, new_buffer));
    }

    pub fn set_indices(&mut self, new_indices: &[u32]) {
        assert!(self.has_vertex, "Setting an index buffer on a pipeline with no vertices!");
        
        //self.context.queue().write_buffer(&vertices.vertex_buffer, 0, bytemuck::cast_slice(new_vertices));
        let new_buffer = self.context.device().create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(new_indices), 
            usage: wgpu::BufferUsages::INDEX
        });
        self.indices = Some((new_indices.len() as u32, new_buffer));
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

    //TODO: think about reusing command encoders for pipelines
    //note: each command encoder needs to have the same frame buffers associated with it
    //a command encoder in wgpu maps onto a render stage
    pub fn render(&self, surface_view: &wgpu::TextureView) {
        let (vertex_count, vertex_buffer) = self.vertices.as_ref().expect("Trying to render without vertices bound");

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

            //this line is controversial
            self.shader.bind_uniforms(&mut render_pass);
            
            //bind our vertices here
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            if let Some((index_count, index_buffer)) = &self.indices {
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..*index_count, 0, 0..1);
            } else {
                render_pass.draw(0..*vertex_count, 0..1);
            }
        }

        self.context.queue().submit(std::iter::once(encoder.finish()));
    }
}

impl RenderPipeline {

    pub fn create(descriptor: RenderPipelineDescriptor, vertex_descriptions: &[wgpu::VertexBufferLayout], api: &RenderApi) -> Self
    {
        let shader = Shader::new(descriptor.shader, api.render_context.clone());

        let uniform_layouts = shader.layouts();
        println!("Shader layouts: {:?}", uniform_layouts);

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
                    cull_mode: None,                //Some(wgpu::Face::Back),
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

            has_vertex: true,
            vertices: None,
            indices: None

        }
    }

}
