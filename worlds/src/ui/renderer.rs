use std::rc::Rc;

use crate::graphics::RenderApi;

use super::TextBox;

pub struct UiRenderer {
    text: Vec<TextBox>,
    render_api: Rc<RenderApi>,
}

impl UiRenderer {
    pub fn add_text(&mut self, text: &str, x: u32, y: u32) {
        self.text.push(TextBox {})
    }
}

impl UiRenderer {
    pub fn new(render_api: Rc<RenderApi>) -> Self {
        Self {
            text: Vec::new(),
            render_api,
        }
    }

    pub fn resize() {}

    pub fn render() {}
}