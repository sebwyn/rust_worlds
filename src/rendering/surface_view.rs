pub struct SurfaceView {
    view: Option<wgpu::SurfaceTexture>,
}

impl SurfaceView {
    pub fn new(view: wgpu::SurfaceTexture) -> Self {
        Self {
            view: Some(view),
        }
    }

    pub fn view(&self) -> Option<wgpu::TextureView> {
        Some(self.view.as_ref()?.texture.create_view(&wgpu::TextureViewDescriptor::default()))
    }

    pub fn present(&mut self) {
        let maybe_view = std::mem::replace(&mut self.view, None);
        if let Some(view) = maybe_view {
            view.present()
        }
    }
}