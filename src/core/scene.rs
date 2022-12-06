use super::Event;

//allows for swapping of scenes, potentially hotswapping!!!
pub trait Scene {
   fn update(&mut self, events: &[Event]);
   fn render(&mut self, surface_view: &wgpu::TextureView);
}
