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
            text: "Hello World".to_string(),
        }
    }

    fn update(&mut self, events: &[Event]) {
        for event in events.iter() {        
            if let Event::KeyPressed(key, _) = event {
                
                if let Some(c) = event.get_character() {
                    self.text.push(c);
                }

                if *key == VirtualKeyCode::Tab {
                    self.text += "    ";
                } else if *key == VirtualKeyCode::Back {
                    self.text.pop();
                }
            }
        }


        self.ui.put_string(&self.text, 0, 300);

        self.ui.update(events);
    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {       
        let mut encoder = render_api.begin_render();

        self.ui.render(surface_view, &mut encoder);

        render_api.end_render(encoder);
    }
}