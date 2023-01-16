use std::mem;

use crate::graphics::Vertex;

#[derive(Clone)]
pub(super) struct Transform2d {
    pub(super) position: [f32; 2],
    pub(super) scale: [f32; 2],
    pub(super) rotation: f32,
}

#[derive(Clone)]
pub(super) struct TextureTransform {
    pub(super) position: [f32; 2],
    pub(super) scale: [f32; 2],
    pub(super) rotation: f32,

    pub(super) tex_position: [f32; 2],
    pub(super) tex_scale: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct TexturedVert { 
    pub position: [f32; 2],
    pub tex_coord: [f32; 2] 
}

impl TexturedVert {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
}

impl Vertex for TexturedVert {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexturedVert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct Vert { 
    pub position: [f32; 2]
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct TexturedInstance {
    model: [[f32; 3]; 3],
    tex_coords: [[f32; 3]; 3],
}

impl From<TextureTransform> for TexturedInstance {
    fn from(transform: TextureTransform) -> Self {
        let rotation_matrix = cgmath::Matrix3::from_angle_z(cgmath::Rad(transform.rotation));
        let scale_matrix = cgmath::Matrix3::from_nonuniform_scale(transform.scale[0], transform.scale[1]);

        let tex_scale_matrix = cgmath::Matrix3::from_nonuniform_scale(transform.tex_scale[0], transform.tex_scale[1]);
        Self {
            model: (cgmath::Matrix3::from_translation(transform.position.into()) * rotation_matrix * scale_matrix).into(),
            tex_coords: (cgmath::Matrix3::from_translation(transform.tex_position.into()) * tex_scale_matrix).into()
        }
    }
}

impl Vertex for TexturedInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexturedInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },

                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 15]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct Instance {
    pub model: [[f32; 3]; 3],
    pub color: [f32; 4],
}

impl Instance {
    pub fn new(transform: Transform2d, color: [f32; 4]) -> Instance {
        let rotation_matrix = cgmath::Matrix3::from_angle_z(cgmath::Rad(transform.rotation));
        let scale_matrix = cgmath::Matrix3::from_nonuniform_scale(transform.scale[0], transform.scale[1]);

        Self {
            model: (cgmath::Matrix3::from_translation(transform.position.into()) * rotation_matrix * scale_matrix).into(),
            color
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
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                }
            ],
        }
    }
}