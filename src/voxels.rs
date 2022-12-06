use crate::graphics::RenderPipeline;
use crate::graphics::RenderApi;
use crate::graphics::UniformBinding;
use crate::graphics::{ShaderDescriptor, RenderPipelineDescriptor, Attachment, AttachmentAccess, RenderPrimitive};

use crate::core::Event;

use std::time::Instant;
use rayon::prelude::*;

use cgmath::One;

//define a vert that just has a position
use crate::graphics::Vertex;
pub use crate::graphics::wgsl_types::{Vec2, Vec3};

fn to_byte_slice(uints: & [u32]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(uints.as_ptr() as *const _, uints.len() * 4)
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vert { 
    pub position: Vec2 
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

struct Voxel {
    exists: bool
}


pub struct Voxels {
    //logic stuff
    start_time: Instant,
    camera_position: Vec3,

    //update this with shit
    view_matrix: cgmath::Matrix4<f32>,
    last_press: (f64, f64),

    //rendering stuff
    pipeline: RenderPipeline,
    camera_position_binding: UniformBinding,
    view_matrix_binding: UniformBinding,
}

impl Voxels {
    const INDICES: [u32; 6] = [ 0, 1, 2, 3, 2, 1];
    const VERTICES: [Vert; 4] = [ 
        Vert { position: Vec2 { x: -1f32, y: -1f32 }},
        Vert { position: Vec2 { x:  1f32, y: -1f32 }},
        Vert { position: Vec2 { x: -1f32, y:  1f32 }},
        Vert { position: Vec2 { x:  1f32, y:  1f32 }},
    ];

    fn generate_voxel(x: u32, y: u32, z: u32) -> Voxel {
        //hardcode a sphere in a 32x32x32 grid

        let sphere_center = Vec3 { x: 16f32, y: 16f32, z: 16f32 };
        let voxel_center = Vec3 {x: x as f32 + 0.5, y: y as f32 + 0.5, z: z as f32 + 0.5 };

        let dist = (sphere_center - voxel_center).magnitude();

        if dist < 8.0 {
            Voxel { exists: true }
        } else {
            Voxel { exists: false }
        }
    }

    pub fn new(api: &RenderApi, width: u32, height: u32) -> Self {
        let mut pipeline = api.create_render_pipeline_with_vertex::<Vert>(RenderPipelineDescriptor { 
            attachment_accesses: vec![
                AttachmentAccess {
                    clear_color: Some([0f64; 4]),
                    attachment: Attachment::Swapchain,
                }
            ], 
            shader: &ShaderDescriptor {
                file: "voxel.wgsl",
            }, 
            primitive: RenderPrimitive::Triangles,
        });

        pipeline.set_vertices(&Self::VERTICES);
        pipeline.set_indices(&Self::INDICES);

        let dimensions: (u32, u32, u32) = (32, 32, 32);

        //generate our voxels, create a texture, and set the texture uniform on the pipeline (no live updating right now)
        let voxels: Vec<u32> = (0..dimensions.1*dimensions.2).into_par_iter().map(|row_pos| {
            let y = row_pos % dimensions.1;
            let z = row_pos / dimensions.1;

            //iterate row_z and construct a u32 with voxel data
            let mut voxel_out = 0u32;
            for x in 0..32 {
                voxel_out |= u32::from(Self::generate_voxel(x, y, z).exists) << x;
            }

            voxel_out
        }).collect();

        //generate a texture from
        let texture = api.create_texture::<u32>(dimensions.0, dimensions.1, wgpu::TextureFormat::R32Uint);
        texture.write_buffer(to_byte_slice(voxels.as_slice()), api.context().queue());
        let voxel_texture_binding = pipeline.shader().get_texture_binding("voxel_data").expect("Can't find voxel shader binding!");
        pipeline.shader().update_texture(&voxel_texture_binding, &texture, None).expect("Failed to set voxels in a texture group!");

        //set our render uniforms
        let resolution_binding = pipeline.shader().get_uniform_binding("resolution").expect("Can't find resolution uniform in voxel shader!");
        pipeline.shader().set_uniform(&resolution_binding, Vec2 { x: width as f32, y: height as f32 }).expect("failed to set uniform resolution");

        let near_binding = pipeline.shader().get_uniform_binding("near").expect("Can't find near uniform in voxel shader!");
        pipeline.shader().set_uniform(&near_binding, 1f32).expect("failed to set uniform near!");

        let camera_position_binding = pipeline.shader().get_uniform_binding("camera_position").expect("Can't find near camera position uniform in voxel shader!");
        let view_matrix_binding = pipeline.shader().get_uniform_binding("view_matrix").expect("Can't find near view direction uniform in voxel shader!");

        let start_time = Instant::now();

        Self {
            start_time,
            camera_position: Vec3 { x: 0f32, y: 0f32, z: 0f32 },
            view_matrix: cgmath::Matrix4::one(),

            last_press: (0.0, 0.0),

            pipeline,
            camera_position_binding,
            view_matrix_binding,
        }
    }

    pub fn update(&mut self, events: &[Event]) {
        //self.camera_position = Vec3 { x: 0f32, y: 16f32, z: 0f32 };
        //add our sin and cosines of time here
        
        let forward = self.view_matrix * cgmath::Vector4 { x: 0.0, y: 0.0, z: 1.0 , w: 0.0 };
        let left    = self.view_matrix * cgmath::Vector4 { x: 1.0, y: 0.0, z: 0.0 , w: 0.0 };
        
        for event in events {
            match event {
                Event::KeyPressed(key) => {
                    use winit::event::VirtualKeyCode;
                    match key {
                        VirtualKeyCode::S => self.camera_position = self.camera_position - Vec3::from(forward),
                        VirtualKeyCode::W => self.camera_position = self.camera_position + Vec3::from(forward),
                        VirtualKeyCode::A => self.camera_position = self.camera_position - Vec3::from(left),
                        VirtualKeyCode::D => self.camera_position = self.camera_position + Vec3::from(left),
                        VirtualKeyCode::J => self.camera_position.y -= 1.0,
                        VirtualKeyCode::K => self.camera_position.y += 1.0,
                        _ => {}
                        
                    }
                },
                Event::KeyReleased(_) => {},
                Event::MousePressed((_, position)) => { self.last_press = *position; },
                Event::MouseReleased((_, position)) => {
                    //calculate our delta, change our camera angle based on this delta
                    let delta = (position.0 - self.last_press.0, position.1 - self.last_press.1);
                    //convert this delta into pixel space 
                    //for now hard code with and height
                    let width = 800.0;
                    let height = 600.0;
                    
                    //this will make a drag to the very edge of the screen rotate you one quarter
                    let y_rotation = delta.0 / (width / 2.0 ) * (std::f64::consts::PI / 4.0);
                    let x_rotation = delta.1 / (height / 2.0) * (std::f64::consts::PI / 4.0);

                    //do our rotation here
                    self.view_matrix = self.view_matrix *
                        cgmath::Matrix4::from_angle_y(cgmath::Rad(y_rotation as f32)) * 
                        cgmath::Matrix4::from_angle_x(cgmath::Rad(x_rotation as f32));
                },
                Event::CursorMoved(_) => {},
                _ => {}
            }

        } 

        //let duration = ((self.start_time.elapsed().as_millis() % 20000) as f64 / 10000f64 * 2f64 * consts::PI) as f32;
        //self.camera_position.x -= 16f32 * duration.sin();
        //self.camera_position.z -= 16f32 * duration.cos();

        //let view_matrix = cgmath::Matrix4::look_at_rh(camera_position.into(), look_at.into(), cgmath::Vector3 { x: 0f32, y: 1f32, z: 0f32 });
        //self.view_matrix = cgmath::Matrix4::from_angle_y(cgmath::Rad::<f32>(duration));
    }

    pub fn render(&mut self, surface_view: &wgpu::TextureView) {
        let view_matrix_data: [[f32; 4]; 4] = self.view_matrix.into();

        self.pipeline.shader().set_uniform(&self.camera_position_binding, self.camera_position).unwrap();
        self.pipeline.shader().set_uniform(&self.view_matrix_binding, view_matrix_data).unwrap();


        self.pipeline.render(surface_view);
    }
}
