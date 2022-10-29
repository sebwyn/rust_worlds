use bevy_ecs::prelude::*;
use imgui::*;

use crate::{graphics::{RenderContext, Subpass}, core::WindowSystem, app::FrameTime};

pub struct UI {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

impl UI {
    pub fn new(world: &mut World) -> Self {
        let render_context = world.get_resource::<RenderContext>().expect("Render Context dependency of UI is not met");
        let window_system = world.get_resource::<WindowSystem>().expect("WindowSystem dependency of UI is not met");

        let hidpi_factor = window_system.window().scale_factor();
        
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window_system.window(), imgui_winit_support::HiDpiMode::Default);

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let render_config = imgui_wgpu::RendererConfig {
            texture_format: render_context.config.format,
            ..Default::default()
        };

        let renderer = imgui_wgpu::Renderer::new(&mut imgui, &render_context.device, &render_context.queue, render_config);
        
        Self {
            context: imgui,
            platform,
            renderer
        }
    }

    pub fn handle_event<T>(&mut self, world: &mut World, event: &winit::event::Event<T>){
        let window_system = world.get_resource::<WindowSystem>().expect("UI lost contact with window");
        self.platform.handle_event(self.context.io_mut(), window_system.window(), event);
    }

    //probably will end up moving this code out of the render cycle
    pub fn render(&mut self, world: &mut World) {
        let render_context = world.get_resource::<RenderContext>().expect("UI lost contact with render context");
        let window_system = world.get_resource::<WindowSystem>().expect("UI lost contact with window");

        self.platform
            .prepare_frame(self.context.io_mut(), window_system.window()).expect("Unable to prepare frame");
        let ui = self.context.frame();

        {
            let window = imgui::Window::new("Resources");
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(&ui, || {
                    if let Some(frame_time) = world.get_resource::<FrameTime>() {
                        ui.text(format!("Frametime: {}", frame_time.0.as_millis()));
                    }
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });
        }

        let surface_texture = render_context.get_surface_texture();

        //we need to create a render pass here
        let mut ui_pass = Subpass::start(surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default()), render_context, wgpu::LoadOp::Load);

        {
            let mut render_pass = (&mut *ui_pass.encoder.as_mut().unwrap()).begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &ui_pass.texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer.render(ui.render(), &render_context.queue, &render_context.device, &mut render_pass).expect("Rendering ui failed");
        };

        render_context.queue.submit(std::iter::once(ui_pass.finish()));
        
    }
}