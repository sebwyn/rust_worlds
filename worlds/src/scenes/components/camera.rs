use std::rc::Rc;

use cgmath::{One, Point3, SquareMatrix, Vector3, Vector4, EuclideanSpace, Zero};
use winit::dpi::PhysicalPosition;

use crate::{
    core::{Event, Window},
};

pub struct Camera {
    position: Vector3<f32>,
    speed: f32,

    rotation_enabled: bool,
    //update this with shit
    y_rotation: f32,
    x_rotation: f32,

    view_matrix: cgmath::Matrix4<f32>,
    combined_matrix: cgmath::Matrix4<f32>,

    window: Rc<Window>,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    pub fn view_matrix(&self) -> &cgmath::Matrix4<f32> {
        &self.view_matrix
    }
    pub fn combined_matrix(&self) -> &cgmath::Matrix4<f32> {
        &self.combined_matrix
    }
    pub fn position(&self) -> [f32; 3] {
        self.position.into()
    }
}

impl Camera {
    pub fn new(window: Rc<Window>) -> Self {
        Self {
            position: Vector3::zero(),
            speed: 0.5,

            rotation_enabled: false,
            x_rotation: 0f32,
            y_rotation: 0f32,
            view_matrix: cgmath::Matrix4::one(),
            combined_matrix: cgmath::Matrix4::one(),
            window,
        }
    }

    pub fn update(&mut self, events: &[Event]) {
        //self.camera_position = Vec3 { x: 0f32, y: 16f32, z: 0f32 };
        //add our sin and cosines of time here
        
        if events.len() == 0 { return }

        let mut transposed_view = self.view_matrix.clone();
        transposed_view.transpose_self();

        let forward = (self.speed * transposed_view
            * cgmath::Vector4 {
                x:  0.0,
                y:  0.0,
                z: -1.0,
                w:  0.0,
            }).xyz();

        let left = (self.speed * transposed_view
            * cgmath::Vector4 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            }).xyz();

        let half_width = self.window.size().0 as f64 / 2.0;
        let half_height = self.window.size().1 as f64 / 2.0;

        for event in events {
            match event {
                Event::KeyPressed(key, _) => {
                    use winit::event::VirtualKeyCode;
                    match key {
                        //handle movement keys
                        VirtualKeyCode::S => self.position = self.position - forward, //[self.position[0] - forward.x, self.position[1] - forward.y, self.position[2] - forward.z],
                        VirtualKeyCode::W => self.position = self.position + forward,
                        VirtualKeyCode::A => self.position = self.position - left,
                        VirtualKeyCode::D => self.position = self.position + left,
                        VirtualKeyCode::J => self.position.y -= 1.0,
                        VirtualKeyCode::K => self.position.y += 1.0,
                        
                        //enable or disable cursor lock
                        VirtualKeyCode::E => {
                            //toggle rotation
                            self.rotation_enabled = !self.rotation_enabled;

                            self.window.winit_window().set_cursor_visible(!self.rotation_enabled);
                            if self.rotation_enabled {
                                self.window
                                    .winit_window()
                                    .set_cursor_position(PhysicalPosition::new(half_width,half_height))
                                    .unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                //handle camera rotation based on cursor movement when locked
                Event::CursorMoved(position) if self.rotation_enabled => {
                    //because cursor grab mode is set we can set cursor position and go from there
                    //do our cursor math here
                    let delta = (position.0 - half_width, position.1 - half_height);

                    //add some angle to our current rotation
                    self.y_rotation += (delta.0 / half_width) as f32;
                    self.x_rotation += (delta.1 / half_height) as f32;
                    self.x_rotation =
                        self.x_rotation.signum() * f32::min(self.x_rotation.abs(), 1.5);

                    self.window
                        .winit_window()
                        .set_cursor_position(Into::<winit::dpi::PhysicalPosition<f64>>::into((
                            half_width,
                            half_height,
                        )))
                        .unwrap();
                }

                //return early if we got no input
                _ => {}
            }
        }

        
        //update our view and combined matrices
        let unit = Vector3 {
            x:  0f32,
            y:  0f32,
            z: -1f32,
        };

        let rotation_matrix = 
              cgmath::Matrix4::from_angle_y(cgmath::Rad(-self.y_rotation))
            * cgmath::Matrix4::from_angle_x(cgmath::Rad(-self.x_rotation));

        let rotated_unit = rotation_matrix * Vector4::new(unit.x, unit.y, unit.z, 0.0);
        let position: Vector3<f32> = self.position.into();
        let rotated_unit: Point3<f32> = Point3::from_vec(rotated_unit.xyz() + position);

        self.view_matrix = cgmath::Matrix4::look_at_rh(
            Point3::from_vec(self.position),
            rotated_unit,
            cgmath::Vector3::unit_y(),
        );

        let perspective = cgmath::perspective(cgmath::Deg(45.0), self.window.size().0 as f32 / self.window.size().1 as f32, 0.1, 100.0);
        self.combined_matrix = OPENGL_TO_WGPU_MATRIX * perspective * self.view_matrix;
    }
}
