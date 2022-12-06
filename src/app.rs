use std::{time::Instant, rc::Rc};
use winit::{event_loop::ControlFlow, event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, Event}};

use crate::graphics::RenderApi;
use crate::core::{Window, EventSystem};

//use crate::RotatingTri;
use crate::Voxels;


pub struct App {
    _width: u32,
    _height: u32,

    window: Rc<Window>,
    api: RenderApi,

    last_frame: Instant,

    events: EventSystem,
    voxels: Voxels,
}

impl App {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        env_logger::init();

        let width = 800;
        let height = 600;

        let window = Rc::new(Window::new(event_loop, "Worlds App", width, height));
        let api = pollster::block_on(RenderApi::new(&window));

        //initialize our event system
        let events = EventSystem::new();

        //initialize our scene
        let voxels = Voxels::new(window.clone(), &api, width, height);

        let last_frame = Instant::now();

        Self {
            window,
            api,

            _width: width,
            _height: height,
            last_frame,

            events,
            voxels
        }
    }

    pub fn update(&mut self) {
        let events = self.events.emit();
        self.voxels.update(&events);
    }

    pub fn render(&mut self) {
        //limit frame rate because this cpu shit is crazy
        let frame_time = self.last_frame.elapsed().as_millis();

        /*
            if frame_time < 50 {
                return
            }
        */
        //println!("frame time: {}", frame_time as u32);

        self.last_frame = Instant::now();

        //update the tex offset to move in a circle
        let current_texture = self.api.surface().get_current_texture().unwrap();
        let current_texture_view = current_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.voxels.render(&current_texture_view);

        current_texture.present();
    }

    //for now we're doing event based updates, when there are no more events we draw to the screen
    pub fn run() {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut app = Self::new(&event_loop);

        event_loop.run(move |event, _, control_flow| { 
            let my_window_id = app.window.winit_window().id(); 
            
            match event {
                Event::WindowEvent { ref event, window_id, }
                if window_id == my_window_id => match event {
                    //quit if they press escape or close the window
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(_new_size) => {},

                    e => { app.events.handle_event(e); },
                },
                Event::RedrawRequested(window_id) if window_id == my_window_id => {
                    app.update();
                    app.render();
                }
                Event::MainEventsCleared => {
                    app.window.winit_window().request_redraw();
                }
                _ => {}
            };
        });
    }
}
