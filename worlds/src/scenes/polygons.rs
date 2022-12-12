use cgmath::One;
use networking::stream::Agent;

use crate::core::ClientEventFactory;
use crate::core::Scene;
use crate::graphics::RenderApi;
use crate::graphics::RenderPipeline;
use crate::graphics::UniformBinding;
use crate::graphics::Vertex;
use crate::graphics::{
    Attachment, AttachmentAccess, RenderPipelineDescriptor, RenderPrimitive, ShaderDescriptor,
};

use client::open_connection;

use crate::core::Event;
use crate::core::Window;

use std::rc::Rc;

//define a vert that just has a position
pub use crate::graphics::wgsl_types::Vec3;

use super::components::Camera;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vert {
    pub position: Vec3,
}

impl Vert {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];
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

pub struct Polygons {
    //logic stuff
    camera: Camera,
    other_transform: Option<app::Transform>,

    //rendering stuff
    pipeline: RenderPipeline,
    view_matrix_binding: UniformBinding,

    event_factory: ClientEventFactory,
    client_agent: Agent<Vec<app::ClientEvent>, app::Snapshot>,
}

impl Polygons {
    const INDICES: [u32; 6] = [0, 1, 2, 3, 2, 1];
    const VERTICES: [Vert; 4] = [
        Vert {
            position: Vec3 {
                x: -1f32,
                y: -1f32,
                z: 0f32,
            },
        },
        Vert {
            position: Vec3 {
                x: 1f32,
                y: -1f32,
                z: 0f32,
            },
        },
        Vert {
            position: Vec3 {
                x: -1f32,
                y: 1f32,
                z: 0f32,
            },
        },
        Vert {
            position: Vec3 {
                x: 1f32,
                y: 1f32,
                z: 0f32,
            },
        },
    ];

    pub fn new(window: Rc<Window>, api: &RenderApi) -> Self {
        let mut pipeline =
            api.create_render_pipeline_with_vertex::<Vert>(RenderPipelineDescriptor {
                attachment_accesses: vec![AttachmentAccess {
                    clear_color: Some([0f64; 4]),
                    attachment: Attachment::Swapchain,
                }],
                shader: &ShaderDescriptor {
                    file: "shaders/polygons.wgsl",
                },
                primitive: RenderPrimitive::Triangles,
            });

        pipeline.set_vertices(&Self::VERTICES);
        pipeline.set_indices(&Self::INDICES);

        let view_matrix_binding = pipeline
            .shader()
            .get_uniform_binding("combined_matrix")
            .expect("Failed to get view matrix uniform from polygon shader");

        let camera = Camera::new(window.clone());

        if let Ok(obj) = tobj::load_obj("cube.obj", &tobj::LoadOptions::default()) {
            let mesh = &obj.0[0].mesh;
            //load positions into verts
            let vertices: Vec<Vert> = mesh.positions.chunks_exact(3).map(|positions| {
                Vert { position: Vec3 { x: positions[0], y: positions[1], z: positions[2] }}
            }).collect();

            pipeline.set_vertices(&vertices);
            pipeline.set_indices(&mesh.indices);
        }

        let event_factory = ClientEventFactory::new(window);
        let client_agent = open_connection::<Vec<app::ClientEvent>, app::Snapshot>("127.0.0.1").expect("Failed to create a client agent");

        Self {
            camera,
            other_transform: None,

            pipeline,
            view_matrix_binding,
            
            event_factory,
            client_agent
        }
    }
}

impl Scene for Polygons {
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        if events.len() > 0 {
            self.client_agent.send_message(events.iter().map(|event| {
                self.event_factory.create(event) 
            }).collect());
        }
        self.camera.update(events);
        
        //also pull events from our client agent here
        if let Some(app::Snapshot(game_objects)) = self.client_agent.get_messages().last() {
            let mut my_transform: Option<app::Transform> = None;

            for object in game_objects {
                match object {
                    app::GameObject::Player { addr, transform } => { 
                        if addr.ip() == self.client_agent.local_addr().ip() 
                        && addr.port() == self.client_agent.local_addr().port() {
                            my_transform = Some(transform.clone()); 
                        } else {
                            self.other_transform = Some(transform.clone());
                        }
                    },
                }
            }

            if let Some(_transform) = my_transform {
                //update local position with this position
            }
        }
    }

    fn render(&mut self, surface_view: &wgpu::TextureView) {
        let model_matrix = if let Some(other_transform) = &self.other_transform {
            //calculate a model matrix from a transform here 
            cgmath::Matrix4::from_translation(other_transform.position.into())
        } else {
            cgmath::Matrix4::one()
        };

        let combined_matrix = self.camera.combined_matrix() * model_matrix;
        let combined_matrix_data: [[f32; 4]; 4] = combined_matrix.clone().into();

        self.pipeline
            .shader()
            .set_uniform(&self.view_matrix_binding, combined_matrix_data)
            .unwrap();

        self.pipeline.render(surface_view);
    }
}
