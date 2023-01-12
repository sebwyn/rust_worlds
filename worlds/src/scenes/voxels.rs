use crate::core::Scene;
use crate::graphics::RenderPipeline;
use crate::graphics::RenderApi;
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
    color: u8
}

pub struct Voxels {
    //logic stuff
    camera: Camera,
    window: Rc<Window>,

    //rendering stuff
    pipeline: RenderPipeline,

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

        Voxel { color: rand::random::<u8>().clamp(1, 255) }

        /*if dist < 8.0 {
            Voxel { color: rand::random::<u8>().clamp(1, 255) }
        } else {
            Voxel { color: 0 }
        }*/
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

        let dimensions: (u32, u32, u32) = (4, 4, 4);

        //generate our voxels, create a texture, and set the texture uniform on the pipeline (no live updating right now)
        let voxels: Vec<u32> = (0..dimensions.1*dimensions.2).into_par_iter().map(|row_pos| {
            let y = row_pos % dimensions.1;
            let z = row_pos / dimensions.1;

            //iterate row_z and construct a u32 with voxel data
            let mut voxel_out = 0u32;
            for x in 0..4 {
                voxel_out |= (u8::from(Self::generate_voxel(x, y, z).color) as u32) << (x * 8);
            }

            voxel_out
        }).collect();

        //load our color palette
        let palette_file = std::fs::read_to_string("resources/palette.txt").unwrap();
        let mut colors = palette_file.lines().flat_map(|(color)| {
            let (r , gba) = color.split_at(2);
            let (g, b) = gba.split_at(2);

            [
                u8::from_str_radix(r, 16).unwrap(),
                u8::from_str_radix(g, 16).unwrap(),
                u8::from_str_radix(b, 16).unwrap(),
                255
            ]
        }).collect::<Vec<u8>>();

        println!("{:?}", colors);
        println!("{:x?}", voxels);

        assert!(colors.len() <= 256 * 4, "The color palette is too long {}", colors.len());

        let padding = vec![0u8; 4 * 256 - colors.len()];
        colors.extend(padding);

        let palette = api.create_texture::<u32>(256, 1, wgpu::TextureFormat::Rgba8UnormSrgb);
        palette.write_buffer(&colors);

        pipeline.shader().update_texture("palette", &palette, None).expect("Failed to load texture onto gpu");

        //generate a texture from
        let texture = api.create_texture::<u32>(dimensions.0, dimensions.1, wgpu::TextureFormat::R32Uint);
        texture.write_buffer(to_byte_slice(voxels.as_slice()));
        pipeline.shader().update_texture("voxel_data", &texture, None).expect("Failed to set voxels in a texture group!");

        //set our render uniforms
        pipeline.shader().set_uniform("resolution", Vec2 { x: width as f32, y: height as f32 }).expect("failed to set uniform resolution");
        pipeline.shader().set_uniform("near", 1f32).expect("failed to set uniform near!");

        let camera = Camera::new(window.clone());

        Self {
            camera,
            window,

            pipeline,
        }
    }
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        self.camera.update(events);
    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {
        self.pipeline.shader().set_uniform("resolution", Vec2 { x: self.window.size().0 as f32, y: self.window.size().1 as f32 }).expect("failed to set uniform resolution");

        let mut transposed_view_matrix = self.camera.view_matrix().clone();
        transposed_view_matrix.transpose_self();
        let view_matrix_data: [[f32; 4]; 4] = transposed_view_matrix.into();

        self.pipeline.shader().set_uniform("camera_position", *self.camera.position()).unwrap();
        self.pipeline.shader().set_uniform("view_matrix", view_matrix_data).unwrap();

        let mut encoder = render_api.begin_render();
        self.pipeline.render(surface_view, &mut encoder);
        render_api.end_render(encoder)
    }
}
