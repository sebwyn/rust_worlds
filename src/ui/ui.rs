use std::sync::{Arc, Mutex};

use winit::window::Window;
use imgui::*;

pub struct UI {
    context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform
}

unsafe impl Send for UI {}

pub struct ThreadableUI(Arc<Mutex<UI>>);

impl UI {
    pub fn new(window: &Window) -> Self {

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
        
        Self {
            context: imgui,
            platform
        }
    }

    //probably will end up moving this code out of the render cycle
    pub fn build_frame(&mut self, window: &Window) -> Ui {
        self.platform
            .prepare_frame(self.context.io_mut(), window);
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

            let window = imgui::Window::new("Hello too");
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text("Frametime Unkown");
                });
        }

        ui
    }
}