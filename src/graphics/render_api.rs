use winit::window::Window;

pub struct Surface {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

impl Surface {
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.config.width = new_size.0;
        self.config.height = new_size.1;
        self.surface.configure(&self.device, &self.config);
    }
}

pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub async fn init_wgpu(window: &Window) -> (Surface, RenderContext) {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
    let surface = unsafe { instance.create_surface(window) };
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
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync, //VSYNC, can get supported modes here
        //alpha_mode: wgpu::CompositeAlphaMode::Auto, //doesn't exist in the version of wgpu we need to use for text rendering
    };
    surface.configure(&device, &config);

    (
        Surface {
            surface,
            config,
        },
        RenderContext {
            device,
            queue
        }
    )
}
