use image::{Rgba, ImageBuffer};

use super::RenderContext;

use std::fs::File;
use std::io::Read;

pub struct Texture {
    pub width: u32,
    pub height: u32,

    format: wgpu::TextureFormat,

    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler
}

impl Texture {
    pub fn format(&self) -> &wgpu::TextureFormat { &self.format }
    pub fn view(&self) -> &wgpu::TextureView { &self.texture_view }
    pub fn sampler(&self) -> &wgpu::Sampler { &self.sampler }
}

impl Texture {
    pub fn new<T>(width: u32, height: u32, pixel: Vec<u8>, render_context: &RenderContext) -> Self {
        let texture_rgba = match pixel.len() {
            4 => { 
                let mut rgba_slice = [0u8; 4];
                rgba_slice.copy_from_slice(&pixel[0..4]);
                let pixel = Rgba::<u8>(rgba_slice);
                ImageBuffer::from_pixel(width, height, pixel)
            }
            _ => panic!("Invalid pixel format")
        };

        Self::create(texture_rgba, (width, height), render_context)
    }

    pub fn load(file_path: &str, render_context: &RenderContext) -> Self {
        
        let t_bytes_vec = load_file_bytes(file_path);
        let texture_bytes = t_bytes_vec.as_slice();
        let texture_image = image::load_from_memory(texture_bytes).unwrap();
        let texture_rgba = texture_image.to_rgba8();

        Self::create(texture_rgba, (texture_image.width(), texture_image.height()), render_context)
    }

    fn create<T>(texture_rgba: ImageBuffer<T, Vec<u8>>, dimensions: (u32, u32), render_context: &RenderContext) -> Self 
    where 
        T: image::Pixel<Subpixel = u8> 
    {
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = render_context.device().create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Most images are stored using sRGB so we need to reflect that here.
            format,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
        });

        render_context.queue().write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &texture_rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(T::CHANNEL_COUNT as u32 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        let texture_view =
            texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = render_context.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            width: dimensions.0,
            height: dimensions.1,
            format,
            
            texture,
            texture_view,
            sampler,
        }
    }
}

fn load_file_bytes(path: &str) -> Vec<u8> {
    let mut f = File::open(&path).expect("no file found");
    let metadata = std::fs::metadata(&path).expect("unable to read metadata");

    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
