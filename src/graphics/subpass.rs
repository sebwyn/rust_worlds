use super::RenderContext;

pub struct Subpass {
    pub texture: wgpu::TextureView,
    pub encoder: Option<wgpu::CommandEncoder>
}

impl Subpass {
    pub fn start(texture: wgpu::TextureView, render_context: &RenderContext, load_op: wgpu::LoadOp<wgpu::Color>) -> Self {
        let mut encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // Clear frame
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        Subpass { texture, encoder: Some(encoder)}
    }

    pub fn finish(&mut self) -> wgpu::CommandBuffer {
        //take ownership of our encoder here, and finish it
        std::mem::replace(&mut self.encoder, None).expect("Trying to finish an invalid subpass").finish()
    }
}