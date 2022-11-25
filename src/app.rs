use std::rc::Rc;

use winit::{event_loop::ControlFlow, event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, Event}};

use crate::core::Window;
//use crate::graphics::Renderer;

use crate::graphics::RenderApi;
use crate::graphics::ShaderDescriptor;
use crate::graphics::{RenderPipelineDescriptor, RenderPrimitive};
use crate::graphics::{Attachment, AttachmentAccess};

pub struct App;

impl App {

    //for now we're doing event based updates, when there are no more events we draw to the screen
    pub fn run() {
        env_logger::init();

        let event_loop = winit::event_loop::EventLoop::new();
        let window = Rc::new(Window::new("Worlds App", &event_loop));

        let api = pollster::block_on(RenderApi::new(&window));

        //create a shader on our renderer
        let shader = Rc::new(api.create_shader(&ShaderDescriptor { file: "basic.wgsl" }));
        //let binding = shader.get_binding("camera").expect("Couldn't find the camera binding????");
        
        let pipeline = api.create_render_pipeline(RenderPipelineDescriptor { 
            attachment_accesses: vec![
                AttachmentAccess { 
                    clear_color: Some([0f64, 0f64, 0f64, 1f64]), 
                    attachment: Attachment::Swapchain 
                }
            ], 
            shader, 
            primitive: RenderPrimitive::Triangles 
        });

        event_loop.run(move |event, _, control_flow| { 
            let my_window_id = window.winit_window().id(); 
            
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
                    WindowEvent::Resized(_new_size) => {},
                    _ => {} 
                },
                Event::RedrawRequested(window_id) if window_id == my_window_id => {
                    let current_texture = api.surface().get_current_texture().unwrap();
                    let current_texture_view = current_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    pipeline.render(&current_texture_view);

                    current_texture.present();

                }
                Event::MainEventsCleared => {
                    window.winit_window().request_redraw();
                }
                _ => {}
            };
        });
    }
}
