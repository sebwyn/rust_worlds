use super::descriptors::{UniformDescriptor, UniformGroupDescriptor, TextureGroupDescriptor, Usages};

use std::collections::HashMap;
use std::ptr;

#[derive(Debug)]
pub struct ReflectedShader {
    pub vs_entry_point: String,
    pub fs_entry_point: String,
    pub uniform_group_descriptors: Vec<UniformGroupDescriptor>,
    pub texture_group_descriptors: Vec<TextureGroupDescriptor>, 
}

pub fn reflect_shader(shader_source: &str) -> ReflectedShader { 
    let module = naga::front::wgsl::parse_str(shader_source)
        .expect("Naga wgsl error");

    let mut reflector = Reflector { module, ..Default::default() };
    reflector.reflect_variables();
    reflector.reflect_entry_points();

    reflector.reflected()
}

#[derive(Default)]
struct Reflector {
    //inputs
    module: naga::Module,

    vs_entry_point: String,
    fs_entry_point: String,

    texture_group_descriptors: Vec<TextureGroupDescriptor>,
    uniform_group_descriptors: Vec<UniformGroupDescriptor>,

    uniform_descriptors: HashMap<naga::Handle<naga::GlobalVariable>, *mut UniformDescriptor>,
}

impl Reflector {
    fn reflected(self) -> ReflectedShader {
        ReflectedShader { 
            vs_entry_point: self.vs_entry_point,
            fs_entry_point: self.fs_entry_point,
            texture_group_descriptors: self.texture_group_descriptors, 
            uniform_group_descriptors: self.uniform_group_descriptors,
        }
    }

    fn reflect_entry_points(&mut self) {
        assert!(self.module.entry_points.len() == 2, 
            "Worlds Shaders only supports 2 entry points (vertex and fragment), {}", self.module.entry_points.len()
        );

        let vs_entry_point = 
            self.module.entry_points.iter().find(|ep| ep.name == "vs_main")
            .expect("Expected vertex shader to be called vs_main");
        let fs_entry_point = 
            self.module.entry_points.iter().find(|ep| ep.name == "fs_main")
            .expect("Expected fragment shader to be called fs_main");

        self.reflect_entry_point(vs_entry_point, Usages::VERTEX);
        self.reflect_entry_point(fs_entry_point, Usages::FRAGMENT);

        self.vs_entry_point = vs_entry_point.name.to_string();
        self.fs_entry_point = fs_entry_point.name.to_string();
    }

    fn reflect_entry_point(&self, entry_point: &naga::EntryPoint, usage: Usages) {
        for expr in entry_point.function.expressions.iter() {
            if let naga::Expression::GlobalVariable(handle) = expr.1 {
                //look this global up to see if it exists in our uniforms
                if let Some(uniform) = self.uniform_descriptors.get(handle) {
                    unsafe {
                        let current_usage = ptr::addr_of_mut!((**uniform).usages).read_unaligned();
                        ptr::addr_of_mut!((**uniform).usages).write_unaligned(current_usage | usage);
                    }
                }
            }
        } 
    }

    fn reflect_variables(&mut self) {
        for variable in self.module.global_variables.iter() {
            let binding = variable.1.binding.as_ref().unwrap();
            let kind = self.module.types.get_handle(variable.1.ty).unwrap();

            //extract this into a uniform variable
            let uniform = UniformDescriptor { 
                name: String::from(variable.1.name.as_ref().unwrap()), 
                binding: binding.binding,
                size: wgsl_primitive_size(&kind.inner),
                usages: Usages::NONE,
            };

            //check if there is already a texture binding for this
            let uniform_group: Option<&mut UniformGroupDescriptor> = 
                self.uniform_group_descriptors.iter_mut().find(|group| group.index == binding.group);
            let texture_group: Option<&mut TextureGroupDescriptor> = 
                self.texture_group_descriptors.iter_mut().find(|group| group.index == binding.group);
            
            let is_texture_uniform =  matches!(kind.inner, 
                naga::TypeInner::Image { .. } | 
                naga::TypeInner::Sampler { .. }
            );

            if is_texture_uniform {
                assert!(uniform_group.is_none(), "Assigning texture to uniform group");
                
                let texture_group = 
                    if let Some(texture_group) = texture_group {
                        texture_group
                    } else {
                        self.texture_group_descriptors.push(
                            TextureGroupDescriptor { 
                                index: binding.group, 
                                ..Default::default() 
                            }
                        );
                        self.texture_group_descriptors.last_mut().unwrap()
                    };

                if let naga::TypeInner::Image { dim, .. } = kind.inner {
                    texture_group.image = Some(uniform);
                    texture_group.dimensions = dim as u32;
                    self.uniform_descriptors.insert(variable.0, texture_group.image.as_mut().unwrap());
                } else if let naga::TypeInner::Sampler { .. } = kind.inner {
                    texture_group.sampler = Some(uniform);
                    self.uniform_descriptors.insert(variable.0, texture_group.sampler.as_mut().unwrap());
                }
            } else {
                assert!(texture_group.is_none(), "Assigning uniform to texture group");

                let uniform_group = 
                    if let Some(uniform_group) = uniform_group {
                        uniform_group
                    } else {
                        self.uniform_group_descriptors.push(
                            UniformGroupDescriptor { 
                                index: binding.group, 
                                ..Default::default() 
                            },
                        );
                        self.uniform_group_descriptors.last_mut().unwrap()
                    };

                if uniform.size.is_none() {
                    panic!("Worlds WGSL: Don't know what to do with variable: {}", uniform.name);
                }
                uniform_group.uniforms.push(uniform);
                self.uniform_descriptors.insert(variable.0, uniform_group.uniforms.last_mut().unwrap());
            }
        }
    }
}

fn wgsl_primitive_size(kind: &naga::TypeInner) -> Option<u32> {
    match kind {
        naga::TypeInner::Scalar { width, .. } => Some(*width as u32),
        naga::TypeInner::Vector { size, width, .. } => Some(*size as u32 * *width as u32),
        naga::TypeInner::Matrix { columns, rows, width } => Some(*columns as u32 * *rows as u32 * *width as u32),
        naga::TypeInner::Atomic { width, .. } => Some(*width as u32),
        naga::TypeInner::Struct { span, .. } => Some(*span),
        _ => None,
    }
}
