pub use super::descriptors::UniformGroupDescriptor;

pub struct UniformGroup {
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffers: Vec<wgpu::Buffer>
}

//could potentially fuck up if there are gaps in the bindings, but I'm not even sure if this is
//legal wgsl code so idk?
impl UniformGroup {
    pub fn new(
        device: &wgpu::Device, 
        shader_name: &str,
        mut descriptor: UniformGroupDescriptor
    ) -> Self {
        //sort our groups by their binding
        descriptor.uniforms.sort_by_key(|uniform| uniform.binding);

        let layout = Self::create_bind_group_layout(device, shader_name, &descriptor);
        let (buffers, bind_group) = Self::create_empty_bind_groups(device, shader_name, &layout, &descriptor);

        Self {
            layout,
            bind_group,
            buffers,
        }
    }

    fn create_bind_group_layout(
        device: &wgpu::Device, 
        shader_name: &str, 
        group: &UniformGroupDescriptor) -> wgpu::BindGroupLayout
    {

        let entries: Vec<wgpu::BindGroupLayoutEntry> = 
            group.uniforms
            .iter()
            .map(|uniform| {
                wgpu::BindGroupLayoutEntry {
                        binding: uniform.binding,
                        visibility: uniform.usages.into(),
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
            }) .collect();

        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &entries,
                label: Some(&format!("{}_{}_layout", shader_name, group.index)),
            }
        )
    }

    fn create_empty_bind_groups(
        device: &wgpu::Device ,
        shader_name: &str, 
        layout: &wgpu::BindGroupLayout,
        group: &UniformGroupDescriptor) -> (Vec<wgpu::Buffer>, wgpu::BindGroup)
    {
        let buffers: Vec<wgpu::Buffer> = 
            group.uniforms
            .iter()
            .map(|uniform| {
                device.create_buffer(&wgpu::BufferDescriptor { 
                    label: Some(&format!("{}_uniform_buffer", uniform.name)), 
                    size: uniform.size.unwrap() as u64, 
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, 
                    mapped_at_creation: false 
                })
            }).collect();

        let bind_group_entries: Vec<wgpu::BindGroupEntry> = 
            group.uniforms
            .iter().enumerate()
            .map(|(index, uniform)| {
                wgpu::BindGroupEntry {
                    binding: uniform.binding,
                    resource: buffers[index].as_entire_binding(),
                }
            }).collect();

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout,
                entries: &bind_group_entries,
                label: Some(&format!("{}_{}_bind_group", shader_name, group.index)),
            }
        );

        (buffers, bind_group)
    }

}
