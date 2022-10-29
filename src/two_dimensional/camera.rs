use bevy_ecs::prelude::*;
use winit::event::MouseButton;
use crate::core::{WindowSystem, Event};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[derive(Component)]
pub struct Camera {
    pub position: (f32, f32),

    pub screen_size: (f32, f32),
    pub scale: f32
}

#[derive(Copy, Clone, Debug)]
pub struct CameraMatrix {
    pub view: [[f32; 4]; 4]
}

unsafe impl bytemuck::Pod for CameraMatrix {}
unsafe impl bytemuck::Zeroable for CameraMatrix {}

impl Camera {
    pub fn new(position: (f32, f32)) -> Self {
        Self {
            position,
            screen_size: (0f32, 0f32),
            scale: 100f32
        }
    }

    pub fn get_matrix(&self) -> CameraMatrix {
        let dimensions = (self.screen_size.0 / self.scale, self.screen_size.1 / self.scale);

        //move this to an update function and update this every frame
        let ortho = cgmath::ortho(
            self.position.0, 
            self.position.0 + dimensions.0/2f32, 
            self.position.1, 
            self.position.1 + dimensions.1/2f32, 1f32, -1f32);
        
        let matrix = OPENGL_TO_WGPU_MATRIX * ortho;
        CameraMatrix { view:  matrix.into() }
    }


    //the camera 2d update system
    pub fn resize(mut cameras: Query<&mut Camera>, window_system: Res<WindowSystem>) {
        for mut camera in cameras.iter_mut() {
            let size = window_system.window().inner_size();
            camera.screen_size = (size.width as f32, size.height as f32);
        }
    }
}

#[derive(Component)]
pub struct CameraController {
    last_cursor_pos: (f32, f32),
    start_camera_pos: (f32, f32),
    pressed: bool,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            last_cursor_pos: (0f32, 0f32),
            start_camera_pos: (0f32, 0f32),
            pressed: false,
        }
    }

    pub fn update(mut cameras: Query<(&mut Camera, &mut CameraController)>, mut reader: EventReader<Event>) {
        //respond to events here for every camera

        for event in reader.iter() {
            for (mut camera, mut cam_controller) in cameras.iter_mut() {
                //do our updating here
                match event {
                    Event::MousePressed((MouseButton::Left, position)) => {
                        cam_controller.last_cursor_pos = (position.0 as f32, position.1 as f32);
                        cam_controller.start_camera_pos = camera.position;
                        cam_controller.pressed = true;
                    },
                    Event::CursorMoved(position) => {
                        if cam_controller.pressed {
                            let position = (position.0 as f32, position.1 as f32);
                            let dpass = ((position.0 - cam_controller.last_cursor_pos.0), (position.1 - cam_controller.last_cursor_pos.1) as f32);
                            let position_translation = (dpass.0 / (2f32 * camera.scale), dpass.1 / (2f32 * camera.scale));

                            camera.position = (cam_controller.start_camera_pos.0 - position_translation.0, cam_controller.start_camera_pos.1 + position_translation.1);
                        }
                    }
                    Event::MouseReleased((MouseButton::Left, ..)) => {
                        //do some calculation for what the new position is
                        cam_controller.pressed = false;
                    },
                    _ => {}
                }
            }
        }   
    }
}