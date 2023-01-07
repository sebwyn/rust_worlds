use std::rc::Rc;

use winit::event::VirtualKeyCode;

use crate::ui::UiRenderer;
use crate::{graphics::RenderApi, core::Scene};

use crate::core::{Event, Window};

pub struct Ui {
    ui: UiRenderer,
    text: String,
}

//allows for swapping of scenes, potentially hotswapping!!!
impl Scene for Ui {
    fn new(window: Rc<Window>, api: &RenderApi) -> Self {
        Ui { 
            ui: UiRenderer::new(api),
            text: "Hello World".to_string()
        }
    }

    fn update(&mut self, events: &[Event]) {
        for event in events.iter() {
            if let Event::KeyPressed(key) = event {
                let code = *key as u8;
                if 10 <= code && code as u8 <= 35 {
                    let ascii = code + 87;
                    self.text.push(ascii as char);
                }
                
                if *key == VirtualKeyCode::Back {
                    self.text.pop();
                }
            }
        }


        for (i, c) in self.text.as_bytes().iter().enumerate() {
            self.ui.push_char(*c as char, i as u32 * 20, 300);
        }
    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {       
        let mut encoder = render_api.begin_render();

        self.ui.render(surface_view, &mut encoder);

        render_api.end_render(encoder);
    }
}