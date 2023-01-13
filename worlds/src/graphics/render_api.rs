use std::rc::Rc;
use std::cell::{Ref, RefCell};

use super::{RenderPipeline, RenderPipelineDescriptor, Vertex, Sampler};
use super::Texture;

use crate::core::Window; 

pub struct Surface {
    pub(super) surface: wgpu::Surface,
    pub(super) config: wgpu::SurfaceConfiguration,
}

pub struct RenderContext {
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
}

pub struct RenderApi {
    pub(super) render_context: Rc<RenderContext>,
    pub(super) surface: Rc<RefCell<Surface>>,
}

impl RenderApi {
    pub fn context(&self) -> &RenderContext { self.render_context.as_ref() }
    pub fn surface(&self) -> Ref<Surface>   { self.surface.borrow()        }
}

impl RenderApi {
    pub fn create_texture<T>(&self, dimensions: (u32, u32, u32), texture_format: wgpu::TextureFormat) -> Texture {
        Texture::new::<T>(dimensions, texture_format, self.render_context.clone())
    }

    pub fn load_texture(&self, file: &str) -> Texture {
        Texture::load(file, self.render_context.clone())
    }

    pub fn create_sampler(&self) -> Sampler {
        Sampler::new(&self.render_context.device)
    }
    
    pub fn create_render_pipeline<V: Vertex>(&self, descriptor: RenderPipelineDescriptor) -> RenderPipeline {
        RenderPipeline::new::<V>(descriptor, self)
    }

    pub fn create_instanced_render_pipeline<V: Vertex, I: Vertex>(&self, descriptor: RenderPipelineDescriptor) -> RenderPipeline {
        RenderPipeline::new_instanced::<V, I>(descriptor, self)
    }
}

impl RenderApi {
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.surface.borrow_mut().config.width = new_size.0;
        self.surface.borrow_mut().config.height = new_size.1;
        self.surface().surface.configure(&self.render_context.device, &self.surface().config);
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.borrow().surface.get_current_texture()
    }

    pub fn begin_render(&self) -> wgpu::CommandEncoder {
        self.render_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
            label: Some("Triangle Encoder")
        })
    }

    pub fn end_render(&self, encoder: wgpu::CommandEncoder) {
        self.render_context.queue.submit(std::iter::once(encoder.finish()));
    }
}

impl RenderApi {
    pub async fn new(window: &Window) -> Self {
        let size = window.size();

        // Possible backends: Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::DX12);
        let surface = unsafe { instance.create_surface(window.winit_window()) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.0,
            height: size.1,
            present_mode: wgpu::PresentMode::AutoVsync,
            //alpha_mode: wgpu::CompositeAlphaMode::Auto, //doesn't exist in the version of wgpu we need to use for text rendering
        };
        surface.configure(&device, &config);

        Self {
            surface: Rc::new(RefCell::new(Surface {
                surface,
                config,
            })),
            render_context: Rc::new(RenderContext {
                device,
                queue
            })
        } 
    }
}
