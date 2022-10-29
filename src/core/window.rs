use bevy_ecs::prelude::*;
use winit::{window::{Window, WindowBuilder}, event_loop::EventLoop};

pub struct WindowSystem {
    window: Window
}

impl WindowSystem {
    pub fn register_system<T>(world: &mut World, title: &str, event_loop: &EventLoop<T>) {
        world.insert_resource(WindowSystem::new(title, event_loop));
    }

    fn new<T>(title: &str, event_loop: &EventLoop<T>) -> Self {
        let window: Window = WindowBuilder::new()
            .with_title(title)
            .build(event_loop)
            .unwrap();
        
        Self {
            window
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}