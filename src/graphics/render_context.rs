use winit::window::Window;

//this is a helper class that will be included by any renderer, so that render contexts dont need to be created in each renderer
pub struct RenderContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    surface_texture: Option<wgpu::SurfaceTexture>,
}

impl RenderContext {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &Window) -> Self {
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
            present_mode: wgpu::PresentMode::Fifo, //VSYNC, can get supported modes here
            //alpha_mode: wgpu::CompositeAlphaMode::Auto, //doesn't exist in the version of wgpu we need to use for text rendering
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,

            surface_texture: None
        }
    }

    pub fn build_surface_texture(&mut self) {
        self.surface_texture = Some(self.surface.get_current_texture().unwrap());
    }

    pub fn get_surface_texture(&self) -> &wgpu::SurfaceTexture {
        & *self.surface_texture.as_ref().unwrap()
    }

    pub fn present(&mut self) {
        let surface = std::mem::replace(&mut self.surface_texture, None).expect("No valid surface!");
        surface.present();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        
    }
}
