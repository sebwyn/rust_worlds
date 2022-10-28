use bevy_ecs::prelude::*;

use crate::rendering::{RenderPass, RenderContext};

use super::ThreadableUI;

pub struct UiPass {
    renderer: imgui_wgpu::Renderer,
}


impl RenderPass for UiPass {
    fn get_name() -> &'static str {
        "UiPass"
    }

    fn get_init_system() -> Box<dyn bevy_ecs::system::System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::init))
    }

    fn get_render_system() -> Box<dyn bevy_ecs::system::System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::render))
    }
}

impl UiPass {
    fn init(commands: Commands, ui: Res<ThreadableUI>, render_context: Res<RenderContext>){
        
    }
    fn render(ui: Res<ThreadableUI>) {
    }
}

