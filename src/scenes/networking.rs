use crate::core::{Scene, Event};

//this scene will test the client side of networking
pub struct Networking {}

impl Networking {
    fn new() -> Self {
        

        Self {}
    }
}

//allows for swapping of scenes, potentially hotswapping!!!
impl Scene for Networking {
   fn update(&mut self, _events: &[Event]) {


   }
   fn render(&mut self, _surface_view: &wgpu::TextureView) {


   }
}
