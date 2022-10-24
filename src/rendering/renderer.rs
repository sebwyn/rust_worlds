use std::{collections::HashMap};

use bevy_ecs::prelude::*;
use winit::window::Window;

use super::{RenderContext, SurfaceView};

pub trait RenderPass {
    fn get_name() -> &'static str;
    fn get_init_system() -> Box<dyn System<In = (), Out = ()>>;
    //the render system needs to write to a command buffer
    fn get_render_system() -> Box<dyn System<In = (), Out = ()>>;
}


pub struct RenderPassContainer {
    name: &'static str,
    init_system: fn() -> Box<dyn System<In = (), Out = ()>>,
    render_system: fn() -> Box<dyn System<In = (), Out = ()>>,
}

pub struct Renderer {
    passes: Vec<RenderPassContainer>,
}

struct RenderOrder(Vec<String>);
pub struct CommandBuffers(pub HashMap<String, wgpu::CommandBuffer>);

impl Renderer {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
        }
    }

    pub async fn init(&mut self, world: &mut World, window: &Window) {
        //construct our initialization schedule from all of the passes
        let mut init = SystemStage::parallel();
        for pass in self.passes.iter() {
            init.add_system((pass.init_system)());
        }

        world.insert_resource(RenderContext::new(window).await);
        init.run(world);
    }

    pub fn render(&mut self, world: &mut World) {
        //create our render command
        let mut render = SystemStage::parallel();
        for pass in self.passes.iter() {
            render.add_system((pass.render_system)());
        }

        //before we begin rendering, insert a resource that has our render order so we can use this when we finish rendering
        world.insert_resource(RenderOrder(
            self.passes
                .iter()
                .map(|rp_container| String::from(rp_container.name))
                .collect(),
        ));

        //schedule
        Schedule::default()
            .add_system_to_stage("Begin render", Self::begin_render)
            .add_stage("Render", render)
            .add_system_to_stage("Finish Render", Self::finish_render)
            .run(world);
    }

    pub fn add_pass<T>(&mut self)
    where
        T: RenderPass,
    {
        let name = T::get_name();
        self.passes.push(RenderPassContainer {
            name,
            init_system: T::get_init_system,
            render_system: T::get_render_system,
        });
    }
}

impl Renderer {
    fn begin_render(mut commands: Commands, render_context: Res<RenderContext>) {
        //create our command buffers object
        let command_buffers: HashMap<String, wgpu::CommandBuffer> = HashMap::new();
        commands.insert_resource(command_buffers);

        //create our view object, move this call into render_context, and handle errors there
        let view = render_context.surface.get_current_texture().unwrap();
        commands.insert_resource(SurfaceView::new(view));
    }

    fn finish_render(
        render_order: Res<RenderOrder>,
        mut command_buffers: ResMut<HashMap<String, wgpu::CommandBuffer>>,
        mut surface_view: ResMut<SurfaceView>,
        render_context: Res<RenderContext>,
    ) {
        //iterate our render order generating a command buffer vector
        let ordered_command_buffers: Vec<wgpu::CommandBuffer> = render_order
            .0
            .iter()
            .map(|name| {
                command_buffers
                    .remove(name)
                    .expect("One of the render passes did not produce a command buffer")
            })
            .collect();
        render_context
            .queue
            .submit(ordered_command_buffers.into_iter());

        surface_view.present();
    }
}
