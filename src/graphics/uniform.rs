use wgpu::util::DeviceExt;

use super::RenderContext;

pub struct Uniform {
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    buffer_size: usize
}

impl Uniform {
    pub fn new<T>(render_context: &RenderContext, binding: u32) -> Self 
    where
        T: bytemuck::Pod
    {
        let buffer_size = std::mem::size_of::<T>();
        let uniform_vec = vec![0u8; buffer_size];

        let uniform_name = std::any::type_name::<T>();

        let buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} uniform", uniform_name)),
                    contents: &uniform_vec,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group_layout =
            render_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some(&format!("{} bind group layout", uniform_name)),
                });

        let bind_group =
            render_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding,
                        resource: buffer.as_entire_binding(),
                    }],
                    label: Some(&format!("{} bind group", uniform_name)),
                });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
            buffer_size
        }
    }

    pub fn set_buffer<T>(&mut self, render_context: &RenderContext, cpu_uniform: T) 
    where
        T: bytemuck::Pod, 
    {
        assert!(std::mem::size_of::<T>() == self.buffer_size);
        render_context.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[cpu_uniform]))
    }
}