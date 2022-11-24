use bevy_ecs::prelude::*;

use crate::core::Window;
use super::{render_api, RenderContext, Surface};

//the renderer can only live as long as the window lives
//this system may change

//a renderer is a container for render passes
pub struct Renderer<'a> {
    surface: Surface,
    render_context: RenderContext,

    window: &'a Window,
}

//the public interface for rendering (it is the rendering api)
impl<'a> Renderer<'a> {
    pub fn new(window: &Window) -> Self {
        let (surface, render_context) = pollster::block_on(render_api::init_wgpu(window));
        Self {
            surface,
            render_context,
            window
        }
    }

    //weird ass result enabling early return from match
    pub fn render(&mut self) -> Result<(), ()> {
        let surface_texture = match self.surface.get_surface_texture() {
            Ok(st) => Ok(st),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                //reconfigure our surface                
                self.surface.resize(self.window.size());
                return Err(())
            }
            _ => panic!("Timed out or don't have enough memory for a surface!") 
        }?;

       //now we have a texture we can render to our swapchain!!!  

       Ok(()) 
    }
}
