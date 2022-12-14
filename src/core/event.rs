use bevy_ecs::prelude::*;
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
    //private storage of shit
    mouse_inside: bool,

    pub cursor_moved: bool,
    pub mouse_pos: (f64, f64)
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            mouse_inside: true,
            cursor_moved: false,
            mouse_pos: (0f64, 0f64)
        }
    }

    pub fn init(&self, world: &mut World) {
        world.insert_resource(Events::<Event>::default());
    }

    pub fn on_event(&mut self, world: &mut World, event: &WindowEvent) {
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

        if let Some(event) = event {
            let mut events = world.get_resource_mut::<Events<Event>>().expect("No events in world? has event system been initialized");
            events.send(event);
        }
    }

    pub fn update(&mut self, world: &mut World) {
        //aggregate cursor moved events here
        if self.cursor_moved {
            let mut events = world.get_resource_mut::<Events<Event>>().expect("No events in world? has event system been initialized");
            events.send(Event::CursorMoved(self.mouse_pos));
            self.cursor_moved = false;
        }

        let mut events = world.get_resource_mut::<Events<Event>>().expect("No events in world?, has event system been initialized");
        events.update();
    }
}
