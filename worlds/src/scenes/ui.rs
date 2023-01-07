use std::rc::Rc;

use crate::ui::UiRenderer;
use crate::{graphics::RenderApi, core::Scene};

use crate::core::{Event, Window};

pub struct Ui {
    renderer: UiRenderer,
}

//allows for swapping of scenes, potentially hotswapping!!!
impl Scene for Ui {
    fn new(window: Rc<Window>, api: &RenderApi) -> Self {
        Ui {}
    }

    fn update(&mut self, events: &[Event]) {

    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {

    }
}