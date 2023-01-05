use super::buffers::Buffers;
use super::{Shader, ShaderDescriptor};
use super::{RenderApi, RenderContext};
use super::Texture;

use std::rc::Rc;

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

    buffers: Buffers,

    instanced: bool,

    context: Rc<RenderContext>
}

impl RenderPipeline {
    pub fn shader(&mut self) -> &mut Shader { &mut self.shader }

    pub fn vertices<V: Vertex> (&mut self, vertices: &[V])  { self.buffers.vertices(vertices, &self.context)   }
    pub fn indices             (&mut self, indices: &[u32]) { self.buffers.indices(indices, &self.context)     }
    pub fn instances<I: Vertex>(&mut self, instances: &[I]) {
        assert!(self.instanced, "Trying to set instances on a pipeline that isn't instanced");
        self.buffers.instances(instances, &self.context) 
    }
}

//temporary
impl RenderPipeline {
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
    pub fn render(&self, surface_view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let color_attachments = self.generate_framebuffer(surface_view);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
            label: Some("Triangle Pass"), 
            color_attachments: &color_attachments, 
            depth_stencil_attachment: None
        });

        render_pass.set_pipeline(&self.pipeline);

        //this line is controversial
        self.shader.bind_uniforms(&mut render_pass);
        
        if let Some(e) = self.buffers.draw(&mut render_pass).err() {
            panic!("{}", e)
        }
    }
}

impl RenderPipeline {
    pub fn new<V: Vertex>(descriptor: RenderPipelineDescriptor, api: &RenderApi) -> Self {
        let vertex_buffer_layouts = &[V::desc()];
        Self::create(descriptor, vertex_buffer_layouts, false, api)
    }

    pub fn new_instanced<V: Vertex, I: Vertex>(descriptor: RenderPipelineDescriptor, api: &RenderApi) -> Self {
        let vertex_buffer_layouts = &[V::desc(), I::desc()];
        println!("{:?}", vertex_buffer_layouts);
        Self::create(descriptor, vertex_buffer_layouts, true, api)
    }

    pub fn create(descriptor: RenderPipelineDescriptor, vertex_buffer_layouts: &[wgpu::VertexBufferLayout], instanced: bool, api: &RenderApi) -> Self
    {
        let shader = Shader::new(descriptor.shader, api.render_context.clone());

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
            api.context().device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &uniform_layouts,
                push_constant_ranges: &[],
            });

        //create our pipeline here
        let pipeline =
            api.context().device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader.module(),
                    entry_point: shader.vs_entry_point(),
                    buffers: vertex_buffer_layouts,
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

            buffers: Buffers::default(),
            instanced,

            context: api.render_context.clone()
        }
    }

}
