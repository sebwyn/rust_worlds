use std::{sync::{Arc, Mutex}, ops::DerefMut};

use bevy_ecs::prelude::*;
use wgpu::util::StagingBelt;

use crate::rendering::{CommandBuffers, RenderContext, RenderPass, SurfaceView};
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};

//update this text pass every frame
struct TextPass {
    bounds: (f32, f32),
    glyph_brush: GlyphBrush<()>,
}

#[derive(Component)]
struct TextBox {
    text: String,
    position: (f32, f32),
    color: [f32; 4],
    scale: f32,
}

impl RenderPass for TextPass {
    fn get_name() -> &'static str {
        "TextPass"
    }
    fn get_render_system() -> Box<dyn System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::render))
    }

    fn get_init_system() -> Box<dyn System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::init_text_render_pass))
    }
}

impl TextPass {
    fn init_text_render_pass(mut commands: Commands, render_context: Res<RenderContext>) {
        // Prepare glyph_brush
        let inconsolata =
            ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf"))
                .expect("Couldn't build a font");

        let glyph_brush = GlyphBrushBuilder::using_font(inconsolata)
            .build(&render_context.device, render_context.surface_format);

        let bounds = (
            render_context.size.width as f32,
            render_context.size.height as f32,
        );

        commands.insert_resource(Self {
            bounds,
            glyph_brush
        });

        let staging_belt = wgpu::util::StagingBelt::new(1024);
        commands.insert_resource(Arc::new(Mutex::new(staging_belt)));
    }

    fn update_uniforms(&mut self, render_context: &RenderContext) {
        let new_bounds = (
            render_context.size.width as f32,
            render_context.size.height as f32,
        );
        self.update(new_bounds);
    }

    fn render(
        mut text_pass: ResMut<TextPass>,
        staging_belt: Res<Arc<Mutex<StagingBelt>>>,
        _text_boxes: Query<&TextBox>,
        render_context: Res<RenderContext>,
        surface_view: Res<SurfaceView>,
        mut command_buffers: ResMut<CommandBuffers>,
    ) {
        text_pass.update_uniforms(render_context.as_ref());

        //let mut staging_belt = text_pass.staging_belt.lock().unwrap();

        // Recall unused staging buffers before doing work, this may not work, this begs the argument for a cleanup render pass function
        //staging_belt.recall();

        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Clear frame
        {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view.view().expect("Invalid view in text render pass"),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.4,
                            g: 0.4,
                            b: 0.4,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        let bounds = text_pass.bounds;

        text_pass.glyph_brush.queue(Section {
            screen_position: (30.0, 30.0),
            bounds,
            text: vec![Text::new("Hello wgpu_glyph!")
                .with_color([0.0, 0.0, 0.0, 1.0])
                .with_scale(40.0)],
            ..Section::default()
        });

        text_pass.glyph_brush.queue(Section {
            screen_position: (30.0, 90.0),
            bounds,
            text: vec![Text::new("Hello wgpu_glyph!")
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_scale(40.0)],
            ..Section::default()
        });

        {
            let mut staging_belt_lock = staging_belt.lock().unwrap();
    

            // Draw the text!
            text_pass
                .glyph_brush
                .draw_queued(
                    &render_context.device,
                    staging_belt_lock.deref_mut(),
                    &mut encoder,
                    &surface_view.view().expect("Somehow getting an invalid view here in text render pass"),
                    bounds.0 as u32,
                    bounds.1 as u32,
                )
                .expect("Draw queued");

            // Submit the work!
            staging_belt_lock.finish();
        }
        

        command_buffers
            .0
            .insert(String::from(Self::get_name()), encoder.finish());
    }
}

impl TextPass {
    fn update(&mut self, bounds: (f32, f32)) {
        self.bounds = bounds;
    }
}
