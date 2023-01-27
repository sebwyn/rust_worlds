use crate::core::{EventSystem, Scene, Window};
use crate::graphics::RenderApi;

use std::{rc::Rc, time::Instant};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

pub struct App<SceneType> {
    window: Rc<Window>,
    api: RenderApi,

    last_frame: Instant,

    events: EventSystem,

    scene: SceneType,
}

impl<SceneType: Scene + 'static> App<SceneType> {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self 
    where 
        SceneType: Scene
    {
        env_logger::init();

        let width = 800;
        let height = 600;

        let window = Rc::new(Window::new(event_loop, "Worlds App", width, height));
        let api = pollster::block_on(RenderApi::new(&window));

        //initialize our event system
        let events = EventSystem::new();

        //initialize our scene
        let scene = SceneType::new(window.clone(), &api);

        let last_frame = Instant::now();

        Self {
            window,
            api,

            last_frame,

            events,
            scene,
        }
    }

    pub fn update(&mut self) {
        let events = self.events.emit();
        self.scene.update(&events);
    }

    fn resize(&mut self, new_size: (u32, u32)) {
        self.api.resize(new_size)
    }

    fn render(&mut self) {
        //limit frame rate because this cpu shit is crazy
        let _frame_time = self.last_frame.elapsed().as_millis();
        //println!("Frame time: {}", _frame_time);

        self.last_frame = Instant::now();

        //update the tex offset to move in a circle
        let current_texture = self.api.get_current_texture().unwrap();
        let current_texture_view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.scene.render(&current_texture_view, &self.api);

        current_texture.present();
    }

    //for now we're doing event based updates, when there are no more events we draw to the screen
    pub fn run() {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut app = Self::new(&event_loop);

        event_loop.run(move |event, _, control_flow| {
            let my_window_id = app.window.winit_window().id();

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
                    WindowEvent::Resized(new_size) => {
                        app.resize((new_size.width, new_size.height));
                        app.events.resize((new_size.width, new_size.height));
                    }
                    e => {
                        app.events.handle_event(e);
                    }
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
