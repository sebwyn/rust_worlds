use std::rc::Rc;
use std::cell::{Ref, RefCell};

use super::{Shader, ShaderDescriptor};
use super::{RenderPipeline, RenderPipelineDescriptor, Vertex};

use crate::core::Window; 

pub struct Surface {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

impl Surface {
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}

pub struct RenderContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl RenderContext {
    pub fn device(&self) -> &wgpu::Device { &self.device }
    pub fn queue(&self) -> &wgpu::Queue   { &self.queue  }
}

pub struct RenderApi {
    pub render_context: Rc<RenderContext>,
    pub surface: Rc<RefCell<Surface>>,
}

impl RenderApi {
    pub fn context(&self) -> &RenderContext { self.render_context.as_ref() }
    pub fn surface(&self) -> Ref<Surface>   { self.surface.borrow()        }
}

impl RenderApi {
    pub async fn new(window: &Window) -> Self {
        let size = window.size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::METAL);
        let surface = unsafe { instance.create_surface(window.winit_window()) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
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
            present_mode: wgpu::PresentMode::AutoNoVsync, //VSYNC, can get supported modes here
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

    pub fn create_shader(&self, descriptor: &ShaderDescriptor) -> Shader {
        Shader::new(descriptor, self.render_context.clone())
    }

    pub fn create_render_pipeline(&self, descriptor: RenderPipelineDescriptor) -> RenderPipeline {
        RenderPipeline::new(descriptor, self)
    }
    
    pub fn create_render_pipeline_with_vertex<T>(&self, descriptor: RenderPipelineDescriptor) -> RenderPipeline 
    where
        T: Vertex
    {
        RenderPipeline::new_with_vertex::<T>(descriptor, self)
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.surface.borrow_mut().config.width = new_size.0;
        self.surface.borrow_mut().config.height = new_size.1;
        self.surface().surface.configure(&self.render_context.device, &self.surface().config);
    }
}
