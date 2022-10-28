use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use bevy_ecs::prelude::*;

use crate::rendering::{RenderContext, RenderPass, Subpass};
use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};

//update this text pass every frame
pub struct TextPass {
    bounds: (f32, f32),
    glyph_brush: GlyphBrush<()>,
    staging_belt: Arc<Mutex<wgpu::util::StagingBelt>>
}

#[derive(Component)]
pub struct TextBox {
    pub text: String,
    pub position: (f32, f32),
    pub color: [f32; 4],
    pub scale: f32,
}

impl RenderPass for TextPass {
    fn get_name() -> &'static str {
        "TextPass"
    }
    fn get_render_system() -> Box<dyn System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::render))
    }

    fn get_init_system() -> Box<dyn System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::init))
    }
}

impl TextPass {
    fn init(mut commands: Commands, render_context: Res<RenderContext>) {
        // Prepare glyph_brush
        let inconsolata =
            ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf"))
                .expect("Couldn't build a font");

        let glyph_brush = GlyphBrushBuilder::using_font(inconsolata)
            .build(&render_context.device, render_context.config.format);

        let bounds = (
            render_context.size.width as f32,
            render_context.size.height as f32,
        );

        let staging_belt = Arc::new(Mutex::new(wgpu::util::StagingBelt::new(1024)));

        commands.insert_resource(Self {
            bounds,
            glyph_brush,
            staging_belt
        });
    }

    fn render(
        text_boxes: Query<&TextBox>,
        mut text_pass: ResMut<TextPass>,
        mut subpass: ResMut<Subpass>,
        render_context: Res<RenderContext>,
    ) {
        {
            let mut staging_belt_lock = text_pass.staging_belt.lock().unwrap();
            staging_belt_lock.recall();
        }

        //we can't reference text pass later, because it is behind a mut reference
        let bounds = text_pass.bounds;

        text_boxes.for_each(|text_box| {
            text_pass.glyph_brush.queue(Section {
                screen_position: text_box.position,
                bounds,
                text: vec![Text::new(&text_box.text)
                    .with_color(text_box.color)
                    .with_scale(text_box.scale)],
                ..Section::default()
            })
        });

        //what this is doing actually makes a lot of sense, but rust is a little stupid
        let text_pass = &mut *text_pass;
        let subpass = &mut *subpass;

        let mut staging_belt_lock = text_pass.staging_belt.lock().unwrap();

        // Draw the text!
        text_pass
            .glyph_brush
            .draw_queued(
                &render_context.device,
                staging_belt_lock.deref_mut(),
                &mut *subpass.encoder.as_mut().expect("Cannot access an invalid subpass"),
                &subpass.texture,
                bounds.0 as u32,
                bounds.1 as u32,
            )
            .expect("Draw queued");

        // Submit the work!
        staging_belt_lock.finish();
    }
}
