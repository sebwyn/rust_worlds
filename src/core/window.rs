use winit::event_loop::EventLoop;
use winit::dpi::PhysicalSize;

pub struct Window {
    window: winit::window::Window,
}

impl Window {
    pub fn new<T>(event_loop: &EventLoop<T>, title: &str, width: u32, height: u32) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(PhysicalSize { width, height })
            .build(event_loop)
            .unwrap();
        
        Window {
            window
        }
    }

    pub fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    pub fn winit_window(&self) -> &winit::window::Window {
        &self.window
    }
}
