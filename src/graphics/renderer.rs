use bevy_ecs::prelude::*;

use crate::core::WindowSystem;

use super::{RenderContext, Subpass};

pub trait RenderPass {
    fn get_name() -> &'static str;
    fn get_init_system() -> Box<dyn System<In = (), Out = ()>>;
    fn get_render_system() -> Box<dyn System<In = (), Out = ()>>;
}

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

    pub async fn init(&mut self, world: &mut World) {
        //window is a dependency of renderer
        let window_system = world.get_resource::<WindowSystem>().expect("WindowSystem dependency of renderer is not met");

        world.insert_resource(RenderContext::new(window_system.window()).await);

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

        let render_context = world.get_resource::<RenderContext>().expect("There should be a render context here");
        let surface_texture = render_context.get_surface_texture();

        //start our renderpass with the data that we need
        let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        world.insert_resource(Subpass::start(texture_view, render_context, wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.1, a: 1.0}) ));

        //begin our pass here
        self.render_schedule.run(world);
    }

    pub fn add_pass<T>(&mut self)
    where
        T: RenderPass,
    {
        self.passes.push(RenderPassContainer {
            name: T::get_name(),
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
