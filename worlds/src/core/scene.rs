use std::rc::Rc;

use crate::graphics::RenderApi;

use super::{Event, Window};

//allows for swapping of scenes, potentially hotswapping!!!
pub trait Scene {
   fn new(window: Rc<Window>, api: &RenderApi) -> Self;
   fn update(&mut self, events: &[Event]);
   fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi);
}
