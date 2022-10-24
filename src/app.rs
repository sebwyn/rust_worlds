use std::time::{Instant, Duration};

use bevy_ecs::prelude::*;
use env_logger::fmt::Color;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

use crate::rendering::RenderContext;

use super::rendering::ClearColorRenderer;

pub struct ColorMask(u32);

pub struct App;

impl App {

    pub fn update(mut last_frame: ResMut<Instant>, mut counter_duration: ResMut<Duration>, mut renderer: ResMut<ClearColorRenderer>, mut color_mask: ResMut<ColorMask>) {
        let elapsed = last_frame.elapsed();
        println!("Update delta: {}", elapsed.as_micros());
        *last_frame = Instant::now();

        *counter_duration += elapsed;
        if counter_duration.as_secs() > 1 {
            *counter_duration = Duration::new(0, 0);

            //change the background color
            color_mask.0 = (color_mask.0 + 1) & 7;
            renderer.set_color((color_mask.0 & 4) as f64, (color_mask.0 & 2) as f64, (color_mask.0 & 1) as f64);
        }
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
        let mut render_schedule = ClearColorRenderer::init_render(&mut world);

        //init our shit
        world.insert_resource(Instant::now());
        world.insert_resource(Duration::new(0, 0));
        world.insert_resource(ColorMask(1));

        world.insert_resource(RenderContext::new(&window).await);
        world.insert_resource(ClearColorRenderer::new(0.0, 0.0, 0.0));

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
                WindowEvent::ScaleFactorChanged { new_inner_size: new_size, .. } => {

                },

                e => {

                }
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                update_stage.run(&mut world);
                render_schedule.run(&mut world);
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
