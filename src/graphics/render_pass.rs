use super::RenderContext;

pub trait UserPass {
    //might want a stage here, instead of a pass
    fn clear_color(&self) -> [f32; 4];
    fn attachments(&self) -> Vec<Attachment>;

    fn render(&self, encoder: &wgpu::CommandEncoder);  
}

pub struct RenderPass {
    pub texture: Vec<wgpu::TextureView>, //attachments
    clear_color: wgpu::Color,

    device: &wgpu::Device, 
}

impl RenderPass {
    pub fn new(device: &wgpu::Device, clear_color: &[f32; 4], ) -> Self {
        Self {
            texture: Vec::new(),
            clear_color: clear_color.into(),  
            device,
        }
    }

    pub fn start(&self) {
        let mut encoder = 
            self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Subpass"),
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
