mod shader_core;
pub use shader_core::{Shader, ShaderDescriptor, UniformBinding, TextureBinding};

mod descriptors;

mod reflector;

mod uniform_group;
use uniform_group::UniformGroup;

mod texture_group;
use texture_group::TextureGroup;

#[cfg(test)]
mod tests;
