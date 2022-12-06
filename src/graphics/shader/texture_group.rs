use super::descriptors::{Usages, TextureGroupDescriptor};
use crate::graphics::{Texture, Sampler};

pub struct TextureGroup {
    name: String,
    texture_binding: u32,
    sampler_binding: Option<u32>,

    pub layout: wgpu::BindGroupLayout,
    pub bind_group: Option<wgpu::BindGroup>
}

impl TextureGroup {
    pub fn name(&self) -> &str { &self.name }
}

//right now this texture group is using defaults later you should be able to set a _layout
//on a texture group
impl TextureGroup {
    pub fn new(device: &wgpu::Device, descriptor: TextureGroupDescriptor) -> Self {

        let image_uniform = descriptor.image.as_ref().expect("No image found for texture group");
        assert!(matches!(image_uniform.usages, Usages::FRAGMENT), "Image uniform is not exclusive to fragment shader!");

        let dimensions = match descriptor.dimensions {
            1 => wgpu::TextureViewDimension::D1,
            2 => wgpu::TextureViewDimension::D2,
            3 => wgpu::TextureViewDimension::D3,
            _ => panic!("Unsupported texture dimensions: {}", image_uniform.name),
        };

        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: image_uniform.binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: dimensions,
                sample_type: wgpu::TextureSampleType::Uint, //TODO: make this pull from whatever
                                                            //type the texture is
            },
            count: None,
        });

        if let Some(sampler_uniform) = descriptor.sampler.as_ref() {
            assert!(matches!(sampler_uniform.usages, Usages::FRAGMENT), "Sampler uniform is not exclusive to fragment shader!");

            entries.push(wgpu::BindGroupLayoutEntry {
                binding: sampler_uniform.binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }


        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &entries,
                label: Some(&format!("{}_layout", image_uniform.name)),
            });

        //could set a default texture here, but idk?
        
        Self {
            name: image_uniform.name.clone(),
            texture_binding: image_uniform.binding,
            sampler_binding: descriptor.sampler.map(|s| s.binding),
            layout,

            bind_group: None,
        }
    }

    pub fn update_texture(&mut self, texture: &Texture, sampler: Option<&Sampler>, device: &wgpu::Device) {
        let mut bind_group_entries = Vec::new();
        
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: self.texture_binding,
            resource: wgpu::BindingResource::TextureView(texture.view()),
        });

        if let Some(sampler) = sampler {
            
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: self.sampler_binding.expect("Expected to set a sampler on a texture bind group that has a sampler defined"),
                resource: wgpu::BindingResource::Sampler(sampler.wgpu_sampler()),
            });
        };

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.layout,
            entries: &bind_group_entries,
            label: Some(&format!("{}_bind_group", self.name)),
        }));
    }
}
