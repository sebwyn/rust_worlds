use bevy_ecs::prelude::*;
use winit::window::Window;

use super::{RenderContext, Subpass};

pub trait RenderPass {
    fn get_name() -> &'static str;
    fn get_init_system() -> Box<dyn System<In = (), Out = ()>>;
    //the render system needs to write to a command buffer
    fn get_render_system() -> Box<dyn System<In = (), Out = ()>>;
}

//strange dyn RenderPass
pub struct RenderPassContainer {
    name: &'static str,
    render_system: fn() -> Box<dyn System<In = (), Out = ()>>,
    init_system: fn() -> Box<dyn System<In = (), Out = ()>>,
}

pub struct Renderer {
    passes: Vec<RenderPassContainer>,
    render_schedule: Schedule,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            render_schedule: Schedule::default(),
        }
    }

    pub async fn init(&mut self, world: &mut World, window: &Window) {
        world.insert_resource(RenderContext::new(window).await);

        let mut init = SystemStage::parallel();
        for pass in self.passes.iter() {
            init.add_system((pass.init_system)());
        }
        init.run(world);

        //we'll have to reconstruct this render schedule everytime we get a new stage

        //from an optimization standpoint, probably want to chain these systems
        for pass in self.passes.iter() {
            self.render_schedule
                .add_stage(pass.name, SystemStage::single((pass.render_system)()));
        }
        self.render_schedule
            .add_stage("End pass", SystemStage::single(Self::finish_render_pass));
    }

    pub fn render(&mut self, world: &mut World) {

        let render_context = world.get_resource::<RenderContext>().expect("Renderer is not initialized and render was called");
        let surface_view = render_context.surface.get_current_texture().unwrap();

        //start our renderpass with the data that we need
        world.insert_resource(Subpass::start(surface_view.texture.create_view(&wgpu::TextureViewDescriptor::default()), render_context));

        //begin our pass here
        self.render_schedule.run(world);

        //present our texture to the surface
        surface_view.present();
    }

    pub fn add_pass<T>(&mut self)
    where
        T: RenderPass,
    {
        //we'll have to call the init system if we've already initialized

        let name = T::get_name();
        self.passes.push(RenderPassContainer {
            name,
            init_system: T::get_init_system,
            render_system: T::get_render_system,
        });
    }
}

impl Renderer {
    fn finish_render_pass(mut subpass: ResMut<Subpass>, render_context: Res<RenderContext>) {
        render_context
            .queue
            .submit(std::iter::once(subpass.finish()));
    }
}
