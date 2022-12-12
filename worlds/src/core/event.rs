use std::rc::Rc;

use app::ClientEvent;
use winit::event::{VirtualKeyCode, MouseButton};

use super::Window;

#[derive(Debug, Clone)]
pub enum Event {
    //input events
    KeyPressed(VirtualKeyCode),
    KeyReleased(VirtualKeyCode),
    MousePressed((MouseButton, (f64, f64))),
    MouseReleased((MouseButton, (f64, f64))),
    CursorMoved((f64, f64)),
}

pub struct ClientEventFactory {
    window: Rc<Window>,
}

impl ClientEventFactory {
    pub fn new(window: Rc<Window>) -> Self {
        Self {
            window,
        }
    }

    fn normalize_position(&self, position: (f64, f64)) -> (f64, f64) {
        let window_size = self.window.winit_window().inner_size(); 
        
        let normalized_x = position.0 / window_size.width as f64;
        let normalized_y = position.1 / window_size.height as f64;
        let centered_x = normalized_x * 2.0 - 1.0; 
        let centered_y = normalized_y * 2.0 - 1.0; 
        (centered_x, centered_y)
    }

    pub fn create(&self, event: &Event) -> ClientEvent {
        match event.clone() {
            Event::KeyPressed(x) => ClientEvent::KeyPressed(x),
            Event::KeyReleased(x) => ClientEvent::KeyReleased(x),
            Event::MousePressed((button, position)) => ClientEvent::MousePressed((button, self.normalize_position(position))),
            Event::MouseReleased((button, position)) => ClientEvent::MouseReleased((button, self.normalize_position(position))),
            Event::CursorMoved(position) => ClientEvent::CursorMoved(self.normalize_position(position)),
        }    
    }
}
