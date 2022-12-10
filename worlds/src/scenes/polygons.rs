use crate::core::Scene;
use crate::graphics::RenderApi;
use crate::graphics::RenderPipeline;
use crate::graphics::UniformBinding;
use crate::graphics::Vertex;
use crate::graphics::{
    Attachment, AttachmentAccess, RenderPipelineDescriptor, RenderPrimitive, ShaderDescriptor,
};

use crate::core::Event;
use crate::core::Window;

use std::rc::Rc;

//define a vert that just has a position
pub use crate::graphics::wgsl_types::Vec3;

use super::shared::Camera;

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

    //rendering stuff
    pipeline: RenderPipeline,
    view_matrix_binding: UniformBinding,
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

        let camera = Camera::new(window);

        Self {
            camera,
            pipeline,
            view_matrix_binding,
        }
    }
}

impl Scene for Polygons {
    //this update just serves as a camera controller right now
    fn update(&mut self, events: &[Event]) {
        self.camera.update(events);
    }

    fn render(&mut self, surface_view: &wgpu::TextureView) {
        let combined_matrix = self.camera.combined_matrix();
        let combined_matrix_data: [[f32; 4]; 4] = combined_matrix.clone().into();

        self.pipeline
            .shader()
            .set_uniform(&self.view_matrix_binding, combined_matrix_data)
            .unwrap();

        self.pipeline.render(surface_view);
    }
}
