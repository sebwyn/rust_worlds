use super::RenderContext;

use std::fs::File;
use std::io::Read;

pub struct Texture {
    extent: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    channel_count: u32,

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
    pub fn new<T>(width: u32, height: u32, render_context: &RenderContext) -> Self 
    where
        T: image::Pixel<Subpixel = u8>
    {
        let channel_count = T::CHANNEL_COUNT as u32;

        let format = match channel_count {
            4 => wgpu::TextureFormat::Rgba8UnormSrgb,
            _ => panic!("Unsupported pixel format")
        };

        Self::create(
            wgpu::TextureFormat::Rgba8UnormSrgb, 
            channel_count,
            (width, height),
            render_context.device()
        )
    }

    pub fn load(file_path: &str, render_context: &RenderContext) -> Self {
        
        let t_bytes_vec = load_file_bytes(file_path);
        let texture_bytes = t_bytes_vec.as_slice();
        let texture_image = image::load_from_memory(texture_bytes).unwrap();
        let texture_rgba = texture_image.to_rgba8();

        let channel_count = 4;

        let texture = Self::create(
            wgpu::TextureFormat::Rgba8UnormSrgb, 
            channel_count,
            (texture_image.width(), texture_image.height()), 
            render_context.device()
        );

        texture.write_image(&texture_rgba, render_context.queue());
        texture
    }

    fn create(format: wgpu::TextureFormat, channel_count: u32, dimensions: (u32, u32), device: &wgpu::Device) -> Self {
        let extent = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            size: extent,
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

        let texture_view =
            texture.create_view(&wgpu::TextureViewDescriptor::default());

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
            extent,
            format,
            channel_count,
            
            texture,
            texture_view,
            sampler,
        }
    }

    pub fn write_image<T>(&self, image: &image::ImageBuffer<T, Vec<u8>>, queue: &wgpu::Queue) 
    where
        T: image::Pixel<Subpixel = u8>
    {
        //add some assertions here to make sure your writing an image with a valid format
        assert!(image.width() == self.extent.width && image.height() == self.extent.height, "Have to write to whole texture!");
        assert!(T::CHANNEL_COUNT as u32 == self.channel_count, "Writing an image to a texture with different channel count!");

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &image,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(self.channel_count * self.extent.width),
                rows_per_image: std::num::NonZeroU32::new(self.extent.height),
            },
            self.extent,
        );
    }
}

fn load_file_bytes(path: &str) -> Vec<u8> {
    let mut f = File::open(&path).expect("no file found");
    let metadata = std::fs::metadata(&path).expect("unable to read metadata");

    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
