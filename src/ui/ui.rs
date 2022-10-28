use std::sync::{Arc, Mutex};

use winit::window::Window;
use imgui::*;

use crate::rendering::{RenderContext, Subpass};

pub struct UI {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

unsafe impl Send for UI {}

pub struct ThreadableUI(pub Arc<Mutex<UI>>);

impl UI {
    pub fn new(window: &Window, render_context: &RenderContext) -> Self {

        let hidpi_factor = window.scale_factor();
        
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, imgui_winit_support::HiDpiMode::Default);

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

    pub fn handle_event<T>(&mut self, window: &Window, event: &winit::event::Event<T>){
        self.platform.handle_event(self.context.io_mut(), window, event);
    }

    //probably will end up moving this code out of the render cycle
    pub fn render(&mut self, window: &Window, render_context: &RenderContext, surface_texture: &wgpu::SurfaceTexture) {
        self.platform
            .prepare_frame(self.context.io_mut(), window).expect("Unable to prepare frame");
        let ui = self.context.frame();

        {
            let window = imgui::Window::new("Hello world");
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text("Hello world!");
                    ui.text("This...is...imgui-rs on WGPU!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            /*let window = imgui::Window::new("Hello too");
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text("Frametime Unkown");
                });*/
        }

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