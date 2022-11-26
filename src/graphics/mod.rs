mod render_api;
//use render_api::RenderContext;
pub use render_api::{RenderApi, Surface, RenderContext};

mod shader;
use shader::Shader;
pub use shader::{ShaderDescriptor, UniformBinding, TextureBinding};

pub mod wgsl_types;

mod attachment;
pub use attachment::{Attachment, AttachmentAccess};

mod render_pipeline;
pub use render_pipeline::{RenderPipeline, RenderPipelineDescriptor, RenderPrimitive, Vertex};

mod renderer;
pub use renderer::Renderer;

mod texture;
pub use texture::Texture;

#[cfg(test)]
mod graphics_tests;
