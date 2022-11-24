use winit::event_loop::EventLoop;

pub struct Window {
    window: winit::window::Window
}

impl Window {
    fn new<T>(title: &str, event_loop: &EventLoop<T>) -> Self {
        let window: Window = winit::window::WindowBuilder::new()
            .with_title(title)
            .build(event_loop)
            .unwrap();
        
        Self {
            window
        }
    }

    pub fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    pub fn winit_window(&self) -> &Window {
        &self.window
    }
}
