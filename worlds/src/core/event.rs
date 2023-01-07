use std::rc::Rc;

use app::ClientEvent;
use winit::event::{VirtualKeyCode, MouseButton};

use super::Window;

#[derive(Debug, Clone)]
pub enum Event {
    //input events
    KeyPressed(VirtualKeyCode, winit::event::ModifiersState),
    KeyReleased(VirtualKeyCode, winit::event::ModifiersState),
    MousePressed((MouseButton, (f64, f64))),
    MouseReleased((MouseButton, (f64, f64))),
    CursorMoved((f64, f64)),
    WindowResized((u32, u32)),
}

impl Event {
    pub fn get_character(&self) -> Option<char> {
        let (key, modifiers) =
            match self {
                Event::KeyPressed(key, modifiers) | 
                Event::KeyReleased(key, modifiers) => { Some((key, modifiers))},
                _ => None
            }?;

        let code = *key as u8;
        let character = if 10 <= code && code as u8 <= 35 {
            let ascii = (code + 87) as char;
            Some((ascii, ascii.to_uppercase().next().unwrap()))
        } else {
            match *key {
                VirtualKeyCode::Key1 => Some(('1', '!')),
                VirtualKeyCode::Key2 => Some(('2', '@')),
                VirtualKeyCode::Key3 => Some(('3', '#')),
                VirtualKeyCode::Key4 => Some(('4', '$')),
                VirtualKeyCode::Key5 => Some(('5', '%')),
                VirtualKeyCode::Key6 => Some(('6', '^')),
                VirtualKeyCode::Key7 => Some(('7', '&')),
                VirtualKeyCode::Key8 => Some(('8', '*')),
                VirtualKeyCode::Key9 => Some(('9', '(')),
                VirtualKeyCode::Key0 => Some(('0', ')')),
                VirtualKeyCode::Space => Some((' ', ' ')),
                VirtualKeyCode::Apostrophe => Some(('\'', '"')),
                VirtualKeyCode::Backslash => Some(('\\', '|')),
                VirtualKeyCode::Comma => Some((',', '<')),
                VirtualKeyCode::Equals => Some(('=', '+')),
                VirtualKeyCode::Grave => Some(('`', '~')),
                VirtualKeyCode::LBracket => Some(('[', '{')),
                VirtualKeyCode::RBracket => Some((']', '}')),
                VirtualKeyCode::Minus => Some(('-', '_')),
                VirtualKeyCode::Period => Some(('.', '>')),
                VirtualKeyCode::Semicolon => Some((';', ':')),
                VirtualKeyCode::Slash => Some(('/', '?')),
                _ => None
            }
        };

        if let Some(c) = character {
            let c = if modifiers.shift() { c.1 } else { c.0 };
            Some(c)
        } else {
            None
        }
    }
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

    pub fn create(&self, event: &Event) -> Option<ClientEvent> {
        match event.clone() {
            Event::KeyPressed(x, _) => Some(ClientEvent::KeyPressed(x)),
            Event::KeyReleased(x, _) => Some(ClientEvent::KeyReleased(x)),
            Event::MousePressed((button, position)) => Some(ClientEvent::MousePressed((button, self.normalize_position(position)))),
            Event::MouseReleased((button, position)) => Some(ClientEvent::MouseReleased((button, self.normalize_position(position)))),
            Event::CursorMoved(position) => Some(ClientEvent::CursorMoved(self.normalize_position(position))),
            _ => None,
        }
    }
}
