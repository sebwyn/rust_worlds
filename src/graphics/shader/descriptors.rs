bitflags! {
    pub struct Usages: u8 {
        const NONE = 0;
        const VERTEX = 1u8; 
        const FRAGMENT = 2u8;
    }
}

impl Into<wgpu::ShaderStages> for Usages {
    fn into(self) -> wgpu::ShaderStages {
        let mut shader_stage = wgpu::ShaderStages::empty();

        if (self & Usages::FRAGMENT) != Usages::NONE {
            shader_stage.toggle(wgpu::ShaderStages::FRAGMENT);
        } 
        if (self & Usages::VERTEX) != Usages::NONE {
            shader_stage.toggle(wgpu::ShaderStages::VERTEX);
        }

        shader_stage
    }
}

impl Default for Usages {
    fn default() -> Self {
        Usages::NONE 
    }
}

#[derive(Clone, Debug)]
pub struct UniformDescriptor {
    pub name: String,
    pub binding: u32,
    pub size: Option<u32>,
    pub usages: Usages,
}

#[derive(Clone, Default, Debug)]
pub struct UniformGroupDescriptor {
    pub index: u32,
    pub uniforms: Vec<UniformDescriptor>,
}

#[derive(Default, Clone, Debug)]
pub struct TextureGroupDescriptor {
    pub index: u32, 
    pub dimensions: u32,
    pub sampler: Option<UniformDescriptor>,
    pub image: Option<UniformDescriptor>,
    pub usages: Usages,
}
