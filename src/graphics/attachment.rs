//an attachment is a render target
pub enum Attachment {
    Swapchain,
    Image((wgpu::TextureFormat, u32, u32)),
    Image3d((wgpu::TextureFormat, u32, u32, u32))
    //depth?
}
