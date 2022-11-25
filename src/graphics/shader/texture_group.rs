use super::descriptors::{Usages, TextureGroupDescriptor};
use crate::graphics::Texture;

pub struct TextureGroup {
    name: String,
    texture_binding: u32,
    sampler_binding: u32,
    pub layout: wgpu::BindGroupLayout,

    bind_group: Option<wgpu::BindGroup>
}

impl TextureGroup {
    pub fn name(&self) -> &str { &self.name }
}

//right now this texture group is using defaults later you should be able to set a _layout
//on a texture group
impl TextureGroup {
    pub fn new(device: &wgpu::Device, descriptor: TextureGroupDescriptor) -> Self {
        let image_uniform = descriptor.image.as_ref().expect("No image found for texture group");
        let sampler_uniform = descriptor.image.as_ref().expect("No sampler found for texture group");

        assert!(matches!(image_uniform.usages, Usages::FRAGMENT), "Image uniform is not exclusive to fragment shader!");
        assert!(matches!(sampler_uniform.usages, Usages::FRAGMENT), "Sampler uniform is not exclusive to fragment shader!");

        let dimensions = match descriptor.dimensions {
            1 => wgpu::TextureViewDimension::D1,
            2 => wgpu::TextureViewDimension::D2,
            3 => wgpu::TextureViewDimension::D3,
            _ => panic!("Unsupported texture dimensions: {}", image_uniform.name),
        };

        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: image_uniform.binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: dimensions,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: sampler_uniform.binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some(&format!("{}_layout", image_uniform.name)),
            });

        //could set a default texture here, but idk?
        
        Self {
            name: image_uniform.name.clone(),
            texture_binding: image_uniform.binding,
            sampler_binding: sampler_uniform.binding,
            layout,

            bind_group: None,
        }
    }

    pub fn update_texture(&mut self, texture: &Texture, device: &wgpu::Device) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: self.texture_binding,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: self.sampler_binding,
                    resource: wgpu::BindingResource::Sampler(texture.sampler()),
                },
            ],
            label: Some(&format!("{}_bind_group", self.name)),
        }));
    }
}
