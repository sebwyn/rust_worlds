use bevy_ecs::prelude::*;

use super::render_context::RenderContext;

pub struct ClearColorRenderer {
    color: wgpu::Color,
}

impl ClearColorRenderer {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            color: wgpu::Color { r, g, b, a: 1.0 }
        }
    }

    pub fn color(&self) -> wgpu::Color {
        self.color
    }

    pub fn set_color(&mut self, r: f64, g: f64, b: f64) {
        self.color = wgpu::Color { r, g, b, a: 1.0 };
    }

    pub fn init_render(world: &mut World) -> Schedule {
        //scaffolding for what this looks like for more complex renderers
        //go through all of the render passes we want for this renderer
        //and generate a stage for each render pass
        
        //add these to the schedule

        Schedule::default().with_stage("Render", SystemStage::parallel().with_system(Self::render))
    }

    fn render(renderer: Res<ClearColorRenderer>, render_context: Res<RenderContext>) {
        //render using our pipeline
        let output = render_context.surface.get_current_texture().unwrap(); //could fail and fuck everything up
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(renderer.color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        render_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        output.present();
    }

}
