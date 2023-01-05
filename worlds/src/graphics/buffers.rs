use wgpu::util::DeviceExt;

use super::{Vertex, RenderContext};

#[derive(Default)]
pub struct Buffers {
    pub(super) vertices: Option<(u32, wgpu::Buffer)>,
    pub(super) indices: Option<(u32, wgpu::Buffer)>,
    pub(super) instances: Option<(u32, wgpu::Buffer)>
}

impl Buffers {
    pub fn vertices<V: Vertex>(&mut self, vertices: &[V], context: &RenderContext) {
        let new_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(vertices), 
            usage: wgpu::BufferUsages::VERTEX
        });
        self.vertices = Some((vertices.len() as u32, new_buffer));
    }

    pub fn indices(&mut self, indices: &[u32], context: &RenderContext) {
        let new_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(indices), 
            usage: wgpu::BufferUsages::INDEX
        });
        self.indices = Some((indices.len() as u32, new_buffer));
    }

    pub fn instances<I: Vertex>(&mut self, instances: &[I], context: &RenderContext) {
        let new_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: None, 
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX
        });
        self.instances = Some((instances.len() as u32, new_buffer));
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) -> Result<(), String> {
        //bind our vertices here
        let vertex_count = * if let Some((count, buffer)) = &self.vertices {
            render_pass.set_vertex_buffer(0, buffer.slice(..));
            Ok(count)
        } else { Err("No Vertices") }?;

        let count = * if let Some((count, buffer)) = &self.instances {
            render_pass.set_vertex_buffer(1, buffer.slice(..));
            Some(count)
        } else { None }.unwrap_or(&1);

        if let Some((index_count, index_buffer)) = &self.indices {
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..*index_count, 0, 0..count);
        } else {
            render_pass.draw(0..vertex_count, 0..count);
        }

        Ok(())
    }
}