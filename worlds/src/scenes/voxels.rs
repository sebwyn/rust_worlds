use crate::core::Scene;
use crate::graphics::RenderPipeline;
use crate::graphics::RenderApi;
use crate::graphics::UniformBinding;
use crate::graphics::{ShaderDescriptor, RenderPipelineDescriptor, Attachment, AttachmentAccess, RenderPrimitive};

use crate::core::Window;
use crate::core::Event;

use std::rc::Rc;
use cgmath::SquareMatrix;

use rayon::prelude::*;

//define a vert that just has a position
use crate::graphics::Vertex;
pub use crate::graphics::wgsl_types::{Vec2, Vec3};

use super::components::Camera;

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
    camera: Camera,

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
}

impl Scene for Voxels {
    fn new(window: Rc<Window>, api: &RenderApi) -> Self {
        let width = window.winit_window().inner_size().width as f32;
        let height = window.winit_window().inner_size().height as f32;

        let mut pipeline = api.create_render_pipeline::<Vert>(RenderPipelineDescriptor { 
            attachment_accesses: vec![
                AttachmentAccess {
                    clear_color: Some([0f64; 4]),
                    attachment: Attachment::Swapchain,
                }
            ], 
            shader: &ShaderDescriptor {
                file: "shaders/voxel.wgsl",
            }, 
            primitive: RenderPrimitive::Triangles,
        });

        pipeline.vertices(&Self::VERTICES);
        pipeline.indices(&Self::INDICES);

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
        texture.write_buffer(to_byte_slice(voxels.as_slice()));
        let voxel_texture_binding = pipeline.shader()
            .get_texture_binding("voxel_data")
            .expect("Can't find voxel shader binding!");
        pipeline.shader().update_texture(&voxel_texture_binding, &texture, None).expect("Failed to set voxels in a texture group!");

        //set our render uniforms
        let resolution_binding = pipeline.shader()
            .get_uniform_binding("resolution")
            .expect("Can't find resolution uniform in voxel shader!");
        pipeline.shader().set_uniform(&resolution_binding, Vec2 { x: width as f32, y: height as f32 }).expect("failed to set uniform resolution");

        let near_binding = pipeline.shader()
            .get_uniform_binding("near")
            .expect("Can't find near uniform in voxel shader!");
        pipeline.shader().set_uniform(&near_binding, 1f32).expect("failed to set uniform near!");

        let camera_position_binding = pipeline.shader()
            .get_uniform_binding("camera_position")
            .expect("Can't find near camera position uniform in voxel shader!");
        let view_matrix_binding = pipeline.shader()
            .get_uniform_binding("view_matrix")
            .expect("Can't find near view direction uniform in voxel shader!");

        let camera = Camera::new(window);

        Self {
            camera,
            pipeline,
            camera_position_binding,
            view_matrix_binding,
        }
    }
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        self.camera.update(events);
    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {
        let mut transposed_view_matrix = self.camera.view_matrix().clone();
        transposed_view_matrix.transpose_self(); 
        let view_matrix_data: [[f32; 4]; 4] = transposed_view_matrix.into();

        self.pipeline.shader().set_uniform(&self.camera_position_binding, *self.camera.position()).unwrap();
        self.pipeline.shader().set_uniform(&self.view_matrix_binding, view_matrix_data).unwrap();

        let mut encoder = render_api.begin_render();
        self.pipeline.render(surface_view, &mut encoder);
        render_api.end_render(encoder)
    }
}
