mod render_api;
//use render_api::RenderContext;
pub use render_api::{RenderApi, Surface, RenderContext};

mod shader;
use shader::Shader;
pub use shader::{ShaderDescriptor, UniformBinding, TextureBinding};

pub mod wgsl_types;

mod render_pipeline;
pub use render_pipeline::{
    RenderPipeline,

    RenderPipelineDescriptor,
    RenderPrimitive,
    Vertex,
    Attachment,
    AttachmentAccess
};

mod texture;
pub use texture::{Texture, Sampler};
