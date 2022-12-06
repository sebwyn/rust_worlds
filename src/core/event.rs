use winit::event::{VirtualKeyCode, WindowEvent, ElementState, MouseButton};

#[derive(Clone)]
pub enum Event {
    KeyPressed(VirtualKeyCode),
    KeyReleased(VirtualKeyCode),
    MousePressed((MouseButton, (f64, f64))),
    MouseReleased((MouseButton, (f64, f64))),
    CursorMoved((f64, f64))
} 

pub struct EventSystem {
    mouse_inside: bool,
    events: Vec<Event>,
    
    pub cursor_moved: bool,
    pub mouse_pos: (f64, f64)
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            mouse_inside: true,
            events: Vec::new(),

            cursor_moved: false,
            mouse_pos: (0f64, 0f64)
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        //construct my event object here
        let event = match event {
            //WindowEvent::Resized(_) => todo!(),
            WindowEvent::KeyboardInput { input, ..} => {
                if let Some(keycode) = input.virtual_keycode {
                    match input.state {
                        winit::event::ElementState::Pressed => {
                            Some(Event::KeyPressed(keycode))
                        },
                        winit::event::ElementState::Released => {
                            Some(Event::KeyReleased(keycode))
                        },
                    }
                } else {
                    None
                }
            },
            //WindowEvent::ModifiersChanged(_) => todo!(),
            WindowEvent::CursorMoved { position, .. } => {
                //update our internal mouse position and don't emit an event
                self.mouse_pos = (position.x, position.y);
                self.cursor_moved = true;
                None
            },
            WindowEvent::CursorEntered { .. } => {
                self.mouse_inside = true;
                None
            },
            WindowEvent::CursorLeft { .. } => {
                self.mouse_inside = false;
                None
            },
            //WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => todo!(),
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => {
                        Some(Event::MousePressed((*button, self.mouse_pos)))
                    }
                    ElementState::Released => {
                        Some(Event::MouseReleased((*button, self.mouse_pos)))
                    }
                }
            },
            _ => None
        };

        //add the event to an internal list of events
        if let Some(event) = event {
            self.events.push(event);
        }
    }

    pub fn emit(&mut self) -> Vec<Event> {
        //aggregate cursor moved events here
        if self.cursor_moved {
            self.events.push(Event::CursorMoved(self.mouse_pos));
            self.cursor_moved = false;
        }
        
        std::mem::take(&mut self.events)
    }
}
