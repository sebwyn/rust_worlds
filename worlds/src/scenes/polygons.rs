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

use std::io::stdin;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;

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

    local_addresses: Vec<Ipv4Addr>,
    event_factory: ClientEventFactory,
    client_agent: Option<Agent<Vec<app::ClientEvent>, app::Snapshot>>,
}

impl Polygons {
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

        println!("What ip would you like to connect to: ");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        let (my_ip, client_agent) = open_connection::<Vec<app::ClientEvent>, app::Snapshot>(&input).expect("Failed to create a client agent");

        let mut local_addresses: Vec<std::net::Ipv4Addr> = local_ip_address::list_afinet_netifas()
            .unwrap()
            .into_iter()
            .filter_map(|(_name, addr)| {
                if let std::net::IpAddr::V4(v4) = addr {
                    Some(v4)
                } else {
                    None
                }
            }).collect();

        println!("{}", my_ip);
        local_addresses.push(Ipv4Addr::from_str(&my_ip).unwrap());

        local_addresses.push(Ipv4Addr::from_str("0.0.0.0").unwrap());

        Self {
            camera,
            other_transform: None,

            pipeline,
            view_matrix_binding,
            
            event_factory,
            client_agent: Some(client_agent),
            local_addresses
        }
    }
}

impl Scene for Polygons {
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        self.camera.update(events);

        if let Some(client_agent) = &self.client_agent {
            if client_agent.lost_connection() {
                //take the client agent
                self.client_agent.take();
                return
            }

            if events.len() > 0 {
                client_agent.send_message(events.iter().map(|event| {
                    self.event_factory.create(event) 
                }).collect());
            }
            
            //also pull events from our client agent here
            if let Some(app::Snapshot(game_objects)) = client_agent.get_messages().last() {
                let mut my_transform: Option<app::Transform> = None;

                //actual ip

                for object in game_objects {
                    match object {
                        app::GameObject::Player { addr, transform } => { 
                            //crazy matching for local address
                            //println!("{}, {:?}", addr, local_addresses);
                            if addr.port() == client_agent.local_addr().port() {
                            if let std::net::IpAddr::V4(v4) = addr.ip() {
                                if self.local_addresses.iter().find(|local_addr| v4 == **local_addr).is_some() {
                                    my_transform = Some(transform.clone()); 
                                    continue;
                                }
                            }}
                            self.other_transform = Some(transform.clone());
                        },
                    }
                }

                if let Some(_transform) = my_transform {
                    //update local position with this position
                }
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
