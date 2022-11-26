use winit::{event_loop::ControlFlow, event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, Event}};
use std::time::Instant;

use crate::core::Window;

use crate::graphics::RenderApi;
use crate::graphics::ShaderDescriptor;
use crate::graphics::{RenderPipelineDescriptor, RenderPrimitive, RenderPipeline};
use crate::graphics::{Attachment, AttachmentAccess};
use crate::graphics::Vertex;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod, Debug)]
struct Vec2 { x: f32, y: f32 }

impl Vec2 {
    fn scale(self, scale: f32) -> Vec2 {
        Vec2 { x: self.x * scale, y: self.y * scale }
    }

    fn magnitude(&self) -> f32 {
        f32::sqrt(self.x.powf(2f32) + self.y.powf(2f32))
    }

    //slow rotation in polar space
    fn rotate(&self, angle: f32) -> Vec2 {
        let magnitude = self.magnitude();
        let current_angle = if self.y > 0f32 {
             f32::acos(self.x / magnitude)
        } else {
            2f32 * std::f32::consts::PI - f32::acos(self.x / magnitude)
        };
        let new_angle = current_angle + angle; 
        let (new_y, new_x) = f32::sin_cos(new_angle);
        Vec2 { x: magnitude * new_x, y: magnitude * new_y }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
struct Vert { position: Vec2, tex_coord: Vec2 }


impl Vert {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
}

impl Vertex for Vert {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct App {
    width: u32,
    height: u32,

    window: Window,
    api: RenderApi,
    pipeline: RenderPipeline,

    positions: Vec<Vec2>,
    start_time: Instant,
}

impl App {
    fn pos_to_tex(pos: Vec2) -> Vec2 {
        Vec2 { x: (pos.x + 1f32) / 2f32, y: (pos.y + 1f32) / 2f32 }
    }

    fn normalize_position(&self, pos: Vec2) -> Vec2 {
        Vec2 { x: pos.x / (self.width as f32/ 2f32), y: pos.y / (self.height as f32 / 2f32)}
    }

    fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        env_logger::init();

        let width = 800;
        let height = 600;

        let window = Window::new(event_loop, "Worlds App", width, height);
        let api = pollster::block_on(RenderApi::new(&window));
        
        let mut pipeline = api.create_render_pipeline_with_vertex::<Vert>(RenderPipelineDescriptor { 
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

        let texture = api.load_texture("tex.jpeg");
        let texture_binding = pipeline.get_texture_binding("diffuse").expect("Can't get texture uniform");
        pipeline.update_texture(&texture_binding, &texture).expect("failed to set texture");

        let magnitude = 120f32;
        let positions: Vec<Vec2> = (0..3).map(|index| index as f32 * (2f32*std::f32::consts::PI) / 3f32).map(|angle| Vec2 { x: magnitude * f32::cos(angle), y: magnitude * f32::sin(angle)}).collect();

        let start_time = Instant::now();

        Self {
            window,
            api,
            pipeline,

            width,
            height,

            positions, 
            start_time,
        }
    }

    pub fn render(&mut self) {
        //update the tex offset to move in a circle
        let angle = 2f32 * std::f32::consts::PI / 200f32;
        let theta = (self.start_time.elapsed().as_millis() % 5000u128) as f32 / 2500f32 * 2f32 * std::f32::consts::PI;
        
        let positions: Vec<Vec2> = self.positions.iter().map(|position| {
            position.rotate(angle)
        }).collect();

        let radius = 60f32;
        let vertices: Vec<Vert> = self.positions.iter().map(|position| {
            let position = self.normalize_position(*position + Vec2 { x:  radius * -f32::cos(theta), y: radius*f32::sin(theta)});
            Vert { position, tex_coord: Self::pos_to_tex(position) }
        }).collect();
        self.pipeline.set_vertices(&vertices);

        self.positions = positions;

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
