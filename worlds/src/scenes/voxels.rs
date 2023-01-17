use crate::core::Scene;
use crate::graphics::RenderPipeline;
use crate::graphics::RenderApi;
use crate::graphics::Texture;
use crate::graphics::{ShaderDescriptor, RenderPipelineDescriptor, Attachment, AttachmentAccess, RenderPrimitive};

use crate::core::Window;
use crate::core::Event;

use std::rc::Rc;
use cgmath::SquareMatrix;

use rayon::prelude::*;

//define a vert that just has a position
use crate::graphics::Vertex;
pub use crate::graphics::wgsl_types::Vec2;

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
    voxels: Vec<u32>,
    voxel_texture: Texture,
    voxel_dimensions: (u32, u32, u32),

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
        let x_dist = x as i32 - 16;
        let y_dist = y as i32 - 16;
        let z_dist = z as i32 - 16;
        let length = (((x_dist * x_dist) + (y_dist * y_dist) + (z_dist * z_dist)) as f64).sqrt();

        if length < 8.0 {
            Voxel { color: rand::random::<u8>().clamp(1, 255) }
        } else {
            Voxel { color: 0 }
        }
    }
    //returns the position in the voxel, and then the offset in the u8
    fn voxel_from_position(&mut self, position: (i32, i32, i32)) -> Option<(&mut u32, u32)> {
        if 0 < position.0 && (position.0 as u32) < self.voxel_dimensions.0 && 
           0 < position.1 && (position.1 as u32) < self.voxel_dimensions.1 && 
           0 < position.2 && (position.2 as u32) < self.voxel_dimensions.2 
        {
            let texture_position = ((position.0 / 2) as u32, (position.1 / 2) as u32, position.2 as u32); //this as u32 is very sus, have to figure out voxel positioning
            let offset: u32 = (position.0 as u32 % 2) | ((position.1 as u32 % 2) << 1);

            let texture_dimensions = (self.voxel_dimensions.0 / 2, self.voxel_dimensions.1 / 2, self.voxel_dimensions.2);
            let chunk_index = texture_position.0 + (texture_position.1 * texture_dimensions.0) + (texture_position.2 * texture_dimensions.0 * texture_dimensions.1);

            let voxel_ref = self.voxels.get_mut(chunk_index as usize).unwrap();
            Some((voxel_ref, offset))
        } else {
            None
        }
    }

    fn cast_ray(&mut self, screen_coords: (u32, u32)) -> (i32, i32, i32) {
        let resolution = (self.window.size().0 as f32, self.window.size().1 as f32);
        let screen_coords = (screen_coords.0 as f32, screen_coords.1 as f32);
        let half_resolution = (resolution.0 / 2.0, resolution.1 / 2.0);
        let near: f32 = 1.0;

        let mut view_matrix = self.camera.view_matrix().clone();
        view_matrix.transpose_self();

        let origin = self.camera.position();

        let p = (-1.0 * (screen_coords.0 - half_resolution.0) / half_resolution.0, (screen_coords.1 - half_resolution.1) / half_resolution.1);
        let screen_world = (p.0, p.1, -near);
        let mag = (screen_world.0.powf(2.0) + screen_world.1.powf(2.0) + screen_world.2.powf(2.0)).sqrt();
        let world_ray = view_matrix * Into::<cgmath::Vector4<f32>>::into([screen_world.0 / mag, screen_world.1 / mag, screen_world.2 / mag, 0.0]);
 
        let step = (world_ray[0].signum() as i32, world_ray[1].signum() as i32, world_ray[2].signum() as i32);
        let t_delta = (1.0 / world_ray[0], 1.0 / world_ray[1], 1.0 / world_ray[2]);

        let world_fract = (origin[0].fract().abs(), origin[1].fract().abs(), origin[2].fract().abs());

        let t_max_x = if world_ray[0] > 0.0 { 1.0 - world_fract.0 } else { world_fract.0 };
        let t_max_y = if world_ray[1] > 0.0 { 1.0 - world_fract.1 } else { world_fract.1 };
        let t_max_z = if world_ray[2] > 0.0 { 1.0 - world_fract.2 } else { world_fract.2 };

        let mut t_max = (t_delta.0 * t_max_x, t_delta.1 * t_max_y, t_delta.2 * t_max_z);

        let mut voxel = (origin[0].floor() as i32, origin[1].floor() as i32, origin[2].floor() as i32);

        //we now have a starting voxel we should check for collision???

        for _ in 0..50 {
            if t_max.0.abs() < t_max.1.abs() && t_max.0.abs() < t_max.2.abs() {
                t_max.0  = t_max.0 + t_delta.0;
                voxel.0 += step.0;
            } else if t_max.1.abs() < t_max.2.abs() {
                t_max.1  = t_max.1 + t_delta.1;
                voxel.1 += step.1;
            } else {
                t_max.2 = t_max.2 + t_delta.2;
                voxel.2 += step.2;
            }

            if let Some((chunk, offset)) = self.voxel_from_position(voxel) {
                let chunk = *chunk;
                let color_index = ((chunk & (0xFF << offset * 8)) >> (offset * 8)) as u8;
                if color_index > 0 {
                    return voxel;
                }
            }
        }

        voxel
    }

    fn create_voxel_texture(dimensions: (u32, u32, u32), api: &RenderApi) -> (Texture, Vec<u32>) {
        let texture_dimensions = (dimensions.0 / 2, dimensions.1 / 2, dimensions.2);

        //generate our voxels, create a texture, and set the texture uniform on the pipeline (no live updating right now)
        let voxels: Vec<u32> = (0..texture_dimensions.0*texture_dimensions.1*texture_dimensions.2).into_par_iter().map(|row_pos| {
            let x= (row_pos % texture_dimensions.0) * 2;
            let y = ((row_pos / texture_dimensions.0) % texture_dimensions.1) * 2;
            let z = row_pos / (texture_dimensions.0 * texture_dimensions.1);

            let mut voxel_out = 0u32;
            for offset in 0..4 {
                let x_offset = offset & 1;
                let y_offset = (offset & 2) >> 1;
                voxel_out |= (Self::generate_voxel(x + x_offset, y + y_offset, z).color as u32) << (offset * 8);
            }
            voxel_out
        }).collect();

        //generate a texture from
        let texture = api.create_texture::<u32>(texture_dimensions, wgpu::TextureFormat::R32Uint);
        texture.write_buffer(to_byte_slice(voxels.as_slice()));

        (texture, voxels)
    }

    fn create_palette_texture(file: &str, api: &RenderApi) -> Texture {
        //load our color palette
        let palette_file = std::fs::read_to_string(file).unwrap();
        let mut colors = palette_file.lines().flat_map(|color| {
            let (r , gba) = color.split_at(2);
            let (g, b) = gba.split_at(2);

            [
                u8::from_str_radix(r, 16).unwrap(),
                u8::from_str_radix(g, 16).unwrap(),
                u8::from_str_radix(b, 16).unwrap(),
                255
            ]
        }).collect::<Vec<u8>>();

        assert!(colors.len() <= 256 * 4, "The color palette is too long {}", colors.len());

        let padding = vec![0u8; 4 * 256 - colors.len()];
        colors.extend(padding);

        let palette = api.create_texture::<u32>((256, 1, 1), wgpu::TextureFormat::Rgba8UnormSrgb);
        palette.write_buffer(&colors);

        palette
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

        let palette = Self::create_palette_texture("resources/palette.txt", api);
        pipeline.shader().update_texture("palette", &palette, None).expect("Failed to load texture onto gpu");


        let voxel_dimensions: (u32, u32, u32) = (32, 32, 32);
        let (voxel_texture , voxels)= Self::create_voxel_texture(voxel_dimensions, api);
        pipeline.shader().update_texture("voxel_data", &voxel_texture, None).expect("Failed to set voxels in a texture group!");

        //set our render uniforms
        pipeline.shader().set_uniform("resolution", Vec2 { x: width as f32, y: height as f32 }).expect("failed to set uniform resolution");
        pipeline.shader().set_uniform("near", 1f32).expect("failed to set uniform near!");

        let camera = Camera::new(window.clone());

        Self {
            camera,
            window,
            voxels,
            voxel_texture,
            voxel_dimensions,

            pipeline,
        }
    }
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        self.camera.update(events);

        let center = (self.window.size().0 / 2, self.window.size().1 / 2); //get the center of the screen

        //ray cast here and get the color
        let looking_at = self.cast_ray(center);
        if let Some((voxel, offset)) = self.voxel_from_position(looking_at) {
            //update the look at square to be some some predetermined color
            *voxel &= !(0xFFu32 << offset * 8);

            self.voxel_texture.write_buffer(to_byte_slice(self.voxels.as_slice()));
            self.pipeline.shader().update_texture("voxel_data", &self.voxel_texture, None).expect("Failed to set voxels in a texture group!");
        }

        println!("{:?}", looking_at);
    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {
        self.pipeline.shader().set_uniform("resolution", Vec2 { x: self.window.size().0 as f32, y: self.window.size().1 as f32 }).expect("failed to set uniform resolution");

        let mut transposed_view_matrix = self.camera.view_matrix().clone();
        transposed_view_matrix.transpose_self();
        let view_matrix_data: [[f32; 4]; 4] = transposed_view_matrix.into();

        self.pipeline.shader().set_uniform("camera_position", self.camera.position()).unwrap();
        self.pipeline.shader().set_uniform("view_matrix", view_matrix_data).unwrap();

        let mut encoder = render_api.begin_render();
        self.pipeline.render(surface_view, &mut encoder);
        render_api.end_render(encoder)
    }
}
