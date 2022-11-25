use std::time::Instant;
use winit::{event_loop::ControlFlow, event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, Event}};

use crate::core::Window;

use crate::graphics::RenderApi;
use crate::graphics::{ShaderDescriptor, UniformBinding};
use crate::graphics::{RenderPipelineDescriptor, RenderPrimitive, RenderPipeline};
use crate::graphics::{Attachment, AttachmentAccess};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct Vec2 { x: f32, y: f32 }

pub struct App {
    window: Window,
    api: RenderApi,
    pipeline: RenderPipeline,
    tex_offset_binding: UniformBinding,
    start_time: Instant,
}

impl App {
    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        env_logger::init();

        let window = Window::new("Worlds App", event_loop);

        let api = pollster::block_on(RenderApi::new(&window));
        
        let mut pipeline = api.create_render_pipeline(RenderPipelineDescriptor { 
            attachment_accesses: vec![
                AttachmentAccess { 
                    clear_color: Some([0f64, 0f64, 0f64, 1f64]), 
                    attachment: Attachment::Swapchain 
                }
            ], 
            shader: &ShaderDescriptor {
                file: "basic.wgsl" 
            }, 
            primitive: RenderPrimitive::Triangles 
        });

        let tex_offset_binding = pipeline.get_uniform_binding("tex_offset").expect("Can't get tex_offset uniform");

        let texture = api.load_texture("tex.jpeg");
        let texture_binding = pipeline.get_texture_binding("diffuse").expect("Can't get texture uniform");
        pipeline.update_texture(&texture_binding, &texture).expect("failed to set texture");

        let start_time = Instant::now();

        Self {
            window,
            api,
            pipeline,
            tex_offset_binding,
            start_time
        }
    }

    pub fn render(&mut self) {
        //update the tex offset to move in a circle
        let elapsed = (self.start_time.elapsed().as_millis() % 5000u128) as f32 / 2500f32 * 2f32 * std::f32::consts::PI;

        self.pipeline.set_uniform(&self.tex_offset_binding, Vec2 { x: f32::cos(elapsed) / 10f32, y: f32::sin(elapsed) / 10f32}).expect("Could not set uniform");

        let current_texture = self.api.surface().get_current_texture().unwrap();
        let current_texture_view = current_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.pipeline.render(&current_texture_view);

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
                    //handle resizes
                    WindowEvent::Resized(_new_size) => {},
                    _ => {} 
                },
                Event::RedrawRequested(window_id) if window_id == my_window_id => {
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
