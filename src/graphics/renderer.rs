use std::rc::Rc;

use crate::core::Window;
use super::RenderApi;

//the renderer can only live as long as the window lives
//this system may change

//a renderer is a container for render passes
pub struct Renderer {
    render_api: RenderApi,
    window: Rc<Window>,
}

//the public interface for rendering (it is the rendering api)
impl Renderer {
    pub fn new(window: Rc<Window>) -> Self {
        let render_api = pollster::block_on(RenderApi::new(window.as_ref()));
        Self {
            render_api,
            window
        }
    }

    //weird ass result enabling early return from match
    pub fn render(&mut self) -> Result<(), ()> {
        //have to use if instead of match, to prevent borrowing mutably something that is already
        //borrowed immutably
        let surface_texture_result = self.render_api.surface().get_current_texture();
        let _surface_texture = if let Ok(st) = surface_texture_result {
            Ok(st)
        } else if let Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) = surface_texture_result {
            self.render_api.resize(self.window.size());
            return Err(())
        } else {
            panic!("Timed out or don't have enough memory for a surface!") 
        }?;

       //now we have a texture we can render to our swapchain!!!  

       Ok(()) 
    }
}
