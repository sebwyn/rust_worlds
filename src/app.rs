use std::time::Instant;

use bevy_ecs::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

use crate::{rendering::Renderer, two_dimensional::text::{TextPass, TextBox}};

pub struct App;

impl App {

    pub fn update(mut last_frame: ResMut<Instant>) {
        let elapsed = last_frame.elapsed();
        println!("Update delta: {}", elapsed.as_micros());
        *last_frame = Instant::now();
    } 


    //for now we're doing event based updates, when there are no more events we draw to the screen
    pub async fn run() {

        env_logger::init();
        let event_loop = EventLoop::new();
        let window: Window = WindowBuilder::new()
            .with_title("Worlds")
            .build(&event_loop)
            .unwrap();

        let mut world = World::new();
        let mut update_stage = SystemStage::parallel().with_system(Self::update);

        //init our shit
        world.insert_resource(Instant::now());

        world.spawn().insert(TextBox { text: String::from("Hello World"), position: (30f32, 30f32), color: [0f32, 0f32, 0f32, 1f32], scale: 40f32 });

        let mut renderer = Renderer::new();
        renderer.add_pass::<TextPass>();

        renderer.init(&mut world, &window).await;
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {

                //quit if they press escape or close the window
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,


                //handle resizes
                WindowEvent::Resized(new_size) => {

                },
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                update_stage.run(&mut world);
                renderer.render(&mut world);
                /*match renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => renderer.resize(None),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }*/
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        });
    }
}
