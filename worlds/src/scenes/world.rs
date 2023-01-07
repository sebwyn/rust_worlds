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

use std::io::stdin;
use std::mem;
use std::rc::Rc;
use std::thread;

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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Instance {
    model: [[f32; 4]; 4],
}

impl From<app::Transform> for Instance {
    fn from(transform: app::Transform) -> Self {
        let rotation_matrix = 
                          cgmath::Matrix4::from_angle_y(cgmath::Rad(-transform.rotation[1]))
                        * cgmath::Matrix4::from_angle_x(cgmath::Rad(-transform.rotation[0]));
        Self {
            model: (cgmath::Matrix4::from_translation(transform.position.into()) * rotation_matrix).into(),
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
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct World {
    //logic stuff
    camera: Camera,
    other_transforms: Vec<app::Transform>,

    //rendering stuff
    pipeline: RenderPipeline,
    view_matrix_binding: UniformBinding,

    event_factory: ClientEventFactory,
    client_agent: Option<Agent<Vec<app::ClientEvent>, app::Snapshot>>,
}

impl Scene for World {
    fn new(window: Rc<Window>, api: &RenderApi) -> Self {
        let mut pipeline =
            api.create_instanced_render_pipeline::<Vert, Instance>(RenderPipelineDescriptor {
                attachment_accesses: vec![AttachmentAccess {
                    clear_color: Some([0f64; 4]),
                    attachment: Attachment::Swapchain,
                }],
                shader: &ShaderDescriptor {
                    file: "shaders/polygons.wgsl",
                },
                primitive: RenderPrimitive::Triangles,
            });

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
            
            pipeline.vertices(&vertices);
            pipeline.indices(&mesh.indices);
        }

        let event_factory = ClientEventFactory::new(window);

        println!("What ip would you like to connect to: ");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let ip = if input.len() == 0 {
            //spawn a server thread
            thread::spawn(|| {
                server::Server::new(30).run().expect("Failed to run the server");
            });
            "127.0.0.1"
            
        } else { &input };
        let client_agent = open_connection::<Vec<app::ClientEvent>, app::Snapshot>(ip).expect("Failed to create a client agent");

        Self {
            camera,
            other_transforms: Vec::new(),

            pipeline,
            view_matrix_binding,
            
            event_factory,
            client_agent: Some(client_agent),
        }
    }
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        self.camera.update(events);

        //networking code
        if let Some(client_agent) = &self.client_agent {
            if client_agent.lost_connection() {
                //take the client agent
                self.client_agent.take();
                return
            }

            if events.len() > 0 {
                client_agent.send_message(events.iter().filter_map(|event| {
                    self.event_factory.create(event)
                }).collect());
            }
            
            //also pull events from our client agent here
            if let Some(app::Snapshot { local_id, player_transforms }) = client_agent.get_messages().pop() {
                println!("{}", player_transforms.len());

                 self.other_transforms = player_transforms
                    .into_iter()
                    .filter_map(|(k, v)| {
                        if local_id == k {
                            None
                        } else {
                            Some(v)
                        }
                    })
                    .collect();
            }
        }
    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {

        let instance_buffer: Vec<Instance> = self.other_transforms.iter().map(|i| {Instance::from(i.clone())}).collect();

        self.pipeline.instances(&instance_buffer);

        let combined_matrix = self.camera.combined_matrix();
        let combined_matrix_data: [[f32; 4]; 4] = combined_matrix.clone().into();

        self.pipeline
            .shader()
            .set_uniform(&self.view_matrix_binding, combined_matrix_data)
            .unwrap();

        let mut encoder = render_api.begin_render();
        self.pipeline.render(surface_view, &mut encoder);
        render_api.end_render(encoder)
    }
}