use cgmath::Matrix3;
use cgmath::One;
use cgmath::Vector2;
use cgmath::Vector3;

use crate::ClientEvent as Event;

#[cfg(feature = "server")]
pub struct Player {
    //attributes
    speed: f32,

    //transform
    position: Vector3<f32>,
    rotation: Vector2<f32>,
    rotation_matrix: Matrix3<f32>,

    //event state (client state)
    rotation_enabled: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            speed: 0.5,

            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector2::new(0.0, 0.0),
            rotation_matrix: Matrix3::one(),
            //assume client window size, this will need to be in our handshake
            rotation_enabled: false,
        }
    }

    pub fn transform(&self) -> crate::packets::Transform {
        crate::packets::Transform {
            position: self.position.into(),
            rotation: self.rotation.into(),
        }
    }

    pub fn update(&mut self, events: Vec<Event>) {

        let forward: Vector3<f32> = self.speed * self.rotation_matrix 
            * cgmath::Vector3 {
                x:  0.0,
                y:  0.0,
                z: -1.0,
            };

        let left: Vector3<f32> = self.speed * self.rotation_matrix
            * cgmath::Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            };

        for event in events {
            match event {
                Event::KeyPressed(key) => {
                    use winit::event::VirtualKeyCode;
                    match key {
                        VirtualKeyCode::S => self.position   -= forward,
                        VirtualKeyCode::W => self.position   += forward,
                        VirtualKeyCode::A => self.position   -= left,
                        VirtualKeyCode::D => self.position   += left,
                        VirtualKeyCode::J => self.position.y -= 1.0,
                        VirtualKeyCode::K => self.position.y += 1.0,
                        VirtualKeyCode::E => { self.rotation_enabled = !self.rotation_enabled; }
                        _ => {}
                    }
                }
                //take in a normalized position
                Event::CursorMoved(position) if self.rotation_enabled => {
                    let position = Vector2::new(position.0 as f32, position.1 as f32);

                    //add some angle to our current rotation
                    self.rotation.y += position.x;
                    self.rotation.x += position.y;
                    self.rotation.x =
                        self.rotation.x.signum() * f32::min(self.rotation.x.abs(), 1.5);

                    self.rotation_matrix = 
                          cgmath::Matrix3::from_angle_y(cgmath::Rad(-self.rotation.y))
                        * cgmath::Matrix3::from_angle_x(cgmath::Rad(-self.rotation.x));
                }
                _ => {}
            }
        }
    }
}

//this will define a client persistent player object that can implement interpolation
#[cfg(feature = "client")]
pub struct CPlayer {
}

impl CPlayer {
}
