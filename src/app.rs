use std::time::{Instant, Duration};

use bevy_ecs::prelude::*;
use winit::{event_loop::ControlFlow, event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, Event}};

#[derive(Debug)]
struct Update;

use crate::{core::WindowSystem, graphics::{Renderer, RenderContext}, two_dimensional::text::{TextPass, TextBox}, ui::UI};

pub struct App;

pub struct FrameTime(pub Duration);

impl App {

    //for now we're doing event based updates, when there are no more events we draw to the screen
    pub async fn run() {

        env_logger::init();
        let event_loop: winit::event_loop::EventLoop<Update> = winit::event_loop::EventLoop::with_user_event();
        //create a proxy, and start another thread
        let update_proxy = event_loop.create_proxy();

        //store the instant
        let last_update_time = Instant::now();
        std::thread::spawn(move || {
            //spawn an update event every 60 seconds
            let elapsed = last_update_time.elapsed();
            if elapsed.as_millis() > (1000f32 / 60f32) as u128 {
                update_proxy.send_event(Update {}).expect("Update thread is exiting");
            }
        });

        //initialize our world
        let mut world = World::new();
        WindowSystem::register_system(&mut world, "Worlds", &event_loop);

        let mut renderer = Renderer::new();
        renderer.add_pass::<TextPass>();
        renderer.init(&mut world).await;

        let mut ui = UI::new(&mut world);

        //init our shit
        world.insert_resource(Instant::now());
        world.spawn().insert(TextBox { text: String::from("Hello World"), position: (30f32, 30f32), color: [0f32, 0f32, 0f32, 1f32], scale: 40f32 });

        let mut last_frame = Instant::now();
        event_loop.run(move |event, _, control_flow| { 
            let my_window_id = world.get_resource::<WindowSystem>().expect("Window does not exist?").window().id();
            
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == my_window_id => match event {
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
                        let mut render_context = world.get_resource_mut::<RenderContext>().expect("Renderer is not initialized and render was called");
                        render_context.resize(*new_size);
                    },
                    _ => {}
                },
                Event::RedrawRequested(window_id) if window_id == my_window_id => {
                    //generate frametime here, and set it as a resource
                    world.insert_resource(FrameTime(last_frame.elapsed()));
                    last_frame = Instant::now();

                    world.get_resource_mut::<RenderContext>().expect("No render context").build_surface_texture();

                    renderer.render(&mut world);
                    ui.render(&mut world);

                    world.get_resource_mut::<RenderContext>().expect("No render context").present();
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
                    world.get_resource::<WindowSystem>().expect("No window?").window().request_redraw();
                }
                _ => {}
            };

            ui.handle_event(&mut world, &event);
        });
    }
}
