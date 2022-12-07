use crate::core::Scene;
use crate::graphics::RenderPipeline;
use crate::graphics::RenderApi;
use crate::graphics::UniformBinding;
use crate::graphics::{ShaderDescriptor, RenderPipelineDescriptor, Attachment, AttachmentAccess, RenderPrimitive};

use crate::core::Window;
use crate::core::Event;

use std::rc::Rc;
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
    camera_position: Vec3,

    rotation_enabled: bool,
    //update this with shit
    y_rotation: f32,
    x_rotation: f32,
    view_matrix: cgmath::Matrix4<f32>,
    window: Rc<Window>,

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

    pub fn new(window: Rc<Window>, api: &RenderApi, width: u32, height: u32) -> Self {
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

        Self {
            camera_position: Vec3 { x: 0f32, y: 0f32, z: 0f32 },

            rotation_enabled: false,
            x_rotation: 0f32,
            y_rotation: 0f32,
            view_matrix: cgmath::Matrix4::one(),
            window,

            pipeline,
            camera_position_binding,
            view_matrix_binding,
        }
    }

}

impl Scene for Voxels {
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        //self.camera_position = Vec3 { x: 0f32, y: 16f32, z: 0f32 };
        //add our sin and cosines of time here
        
        let forward = self.view_matrix * cgmath::Vector4 { x: 0.0, y: 0.0, z: 1.0 , w: 0.0 };
        let left    = self.view_matrix * cgmath::Vector4 { x: 1.0, y: 0.0, z: 0.0 , w: 0.0 };

        let half_width = self.window.size().0 as f64 / 2.0;
        let half_height = self.window.size().1 as f64 / 2.0;
        
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
                        VirtualKeyCode::E => {
                            //toggle rotation
                            self.rotation_enabled = !self.rotation_enabled;

                            if self.rotation_enabled {
                                self.window.winit_window()
                                    .set_cursor_position(Into::<winit::dpi::PhysicalPosition<f64>>::into(
                                            (half_width, half_height)
                                    ))
                                    .unwrap();
                            }
                        },
                        _ => {}
                        
                    }
                },
                Event::KeyReleased(_) => {},
                Event::MousePressed((_, _position)) => {},
                Event::MouseReleased((_, _position)) => {},
                Event::CursorMoved(position) if self.rotation_enabled => {
                    //because cursor grab mode is set we can set cursor position and go from there
                    //do our cursor math here 
                    let delta = (position.0 - half_width, position.1 - half_height);
                    
                    //add some angle to our current rotation
                    self.y_rotation += (delta.0 / half_width) as f32;
                    self.x_rotation += (delta.1 / half_height) as f32;
                    self.x_rotation = self.x_rotation.signum() * f32::min(self.x_rotation.abs(), 1.5);

                    //do our rotation here
                    self.view_matrix =
                        cgmath::Matrix4::from_angle_y(cgmath::Rad(self.y_rotation)) * 
                        cgmath::Matrix4::from_angle_x(cgmath::Rad(self.x_rotation));
                    
                    self.window.winit_window()
                        .set_cursor_position(Into::<winit::dpi::PhysicalPosition<f64>>::into(
                                (half_width, half_height)
                        ))
                        .unwrap();
                },
                _ => {}
            }

        } 
    }

    fn render(&mut self, surface_view: &wgpu::TextureView) {
        let view_matrix_data: [[f32; 4]; 4] = self.view_matrix.into();

        self.pipeline.shader().set_uniform(&self.camera_position_binding, self.camera_position).unwrap();
        self.pipeline.shader().set_uniform(&self.view_matrix_binding, view_matrix_data).unwrap();


        self.pipeline.render(surface_view);
    }


}
