mod render_api;
use render_api::RenderContext;
pub use render_api::RenderApi;

mod shader;
pub use shader::{Shader, ShaderDescriptor};

mod attachment;
pub use attachment::{Attachment, AttachmentAccess};

mod render_pipeline;
pub use render_pipeline::{RenderPipeline, RenderPipelineDescriptor, RenderPrimitive, Vertex};

mod renderer;
pub use renderer::Renderer;

mod render_pass;
pub use render_pass::RenderPass;

mod texture;
pub use texture::Texture;

#[cfg(test)]
mod graphics_tests;
