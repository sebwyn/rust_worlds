use super::RenderContext;

use std::{mem::size_of, rc::Rc};

use crate::util::files;

pub struct Sampler {
    sampler: wgpu::Sampler
}

impl Sampler {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            sampler
        }
    }

    pub fn wgpu_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

pub struct Texture {
    extent: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    bytes_per_pixel: u32,

    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    render_context: Rc<RenderContext>
}

//think about if i want to make this abstract, with images, and buffers or some variation
impl Texture {
    pub fn format(&self) -> &wgpu::TextureFormat { &self.format }
    pub fn view(&self) -> &wgpu::TextureView { &self.texture_view }
}

impl Texture {
    //TODO: create a grpahics texture object, to wrap wgpu
    pub fn new<T>(width: u32, height: u32, format: wgpu::TextureFormat, render_context: Rc<RenderContext>) -> Self {
        let bytes_per_pixel = size_of::<T>() as u32;

        Self::create(
            format,
            bytes_per_pixel,
            (width, height),
            render_context,
        )
    }

    pub fn load(file_path: &str, render_context: Rc<RenderContext>) -> Self {
        
        let t_bytes_vec = files::load_file_bytes(file_path);
        let texture_image = image::load_from_memory(&t_bytes_vec).unwrap();
        let texture_rgba = texture_image.to_rgba8();

        let bytes_per_pixel = 4;

        let texture = Self::create(
            wgpu::TextureFormat::Rgba8UnormSrgb, 
            bytes_per_pixel,
            (texture_image.width(), texture_image.height()), 
            render_context
        );

        texture.write_buffer(&texture_rgba);
        texture
    }

    fn create(format: wgpu::TextureFormat, bytes_per_pixel: u32, dimensions: (u32, u32), render_context: Rc<RenderContext>) -> Self {
        let wgpu_dimensions = match dimensions.1 {
            1 => wgpu::TextureDimension::D1,
            _ => wgpu::TextureDimension::D2,
        };
        
        let extent = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = render_context.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu_dimensions,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
        });

        let texture_view =
            texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            extent,
            format,
            bytes_per_pixel,
            
            texture,
            texture_view,
            render_context
        }
    }

    pub fn write_buffer(&self, buffer: &[u8]) {
        self.render_context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(self.bytes_per_pixel * self.extent.width),
                rows_per_image: std::num::NonZeroU32::new(self.extent.height),
            },
            self.extent,
        );
    }
}