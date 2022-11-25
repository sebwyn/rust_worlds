use super::Attachment;

pub trait UserPass {
    //might want a stage here, instead of a pass
    fn clear_color(&self) -> [f32; 4];
    fn attachments(&self) -> Vec<Attachment>;

    fn render(&self, encoder: &wgpu::CommandEncoder);  
}

pub struct RenderPass<'a> {
    pub texture: Vec<wgpu::TextureView>, //attachments
    clear_color: wgpu::Color,

    device: &'a wgpu::Device, 
}

impl<'a> RenderPass<'a> {
    pub fn new(device: &'a wgpu::Device, clear_color: [f64; 4]) -> Self {
        Self {
            texture: Vec::new(),
            clear_color: wgpu::Color { 
                r: clear_color[0], 
                g: clear_color[1], 
                b: clear_color[2], 
                a: clear_color[3] 
            },  
            device,
        }
    }

    pub fn start<'b>(&self, encoder: &'b mut wgpu::CommandEncoder) -> wgpu::RenderPass<'b> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Subpass"),
            color_attachments: &[],
            depth_stencil_attachment: None,
        })
    }
}
