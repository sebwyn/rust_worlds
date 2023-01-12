use super::descriptors::{UniformDescriptor, UniformGroupDescriptor, TextureGroupDescriptor, GroupDescriptor, Usages, TextureKind};

use std::collections::HashMap;
use std::ptr;

#[derive(Debug)]
pub struct ReflectedShader {
    pub vs_entry_point: String,
    pub fs_entry_point: String,
    pub group_descriptors: Vec<GroupDescriptor>,
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
    
    groups: Vec<(u32, GroupDescriptor)>,

    uniform_descriptors: HashMap<naga::Handle<naga::GlobalVariable>, *mut UniformDescriptor>,
}

impl Reflector {
    //construct our ordered groups, and validate that there are no gaps
    fn reflected(mut self) -> ReflectedShader {
        self.groups.sort_by_key(|(index, _)| *index);
        let group_descriptors = self.groups.into_iter().enumerate()
            .map(|(i, (index, group))| {
                if i as u32 != index {
                    panic!("missing bind group {}", i)
                }
                group
            })
            .collect();

        ReflectedShader { 
            vs_entry_point: self.vs_entry_point,
            fs_entry_point: self.fs_entry_point,
            group_descriptors 
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
            let group: Option<&mut GroupDescriptor> = 
                self.groups.iter_mut().find(|(index, _)| binding.group == *index).map(|(_, group)| group);
            
            let is_texture_uniform =  matches!(kind.inner, 
                naga::TypeInner::Image { .. } | 
                naga::TypeInner::Sampler { .. }
            );

            if is_texture_uniform {
                //get or insert texture descriptor or panic if its not a texture descriptor
                let group: &mut GroupDescriptor = 
                    if matches!(group, Some(GroupDescriptor::Texture(..))) {
                        group.unwrap()
                    } else if group.is_none() {
                        self.groups.push((
                            binding.group,
                            GroupDescriptor::Texture(
                                TextureGroupDescriptor { 
                                    index: binding.group, 
                                    ..Default::default() 
                                }
                            )
                        ));
                        &mut self.groups.last_mut().unwrap().1
                    } else {
                        panic!("Assigning texture to uniform group")
                    };

                if let GroupDescriptor::Texture(texture_group) = group {
                    if let naga::TypeInner::Image { dim, class , ..} = kind.inner {
                        texture_group.image = Some(uniform);
                        match dim {
                            naga::ImageDimension::D1 => texture_group.dimensions = 1,
                            naga::ImageDimension::D2 => texture_group.dimensions = 2,
                            naga::ImageDimension::D3 => texture_group.dimensions = 3,
                            _ => panic!("Cube maps are not supported")
                            //naga::ImageDimension::Cube => texture_group.dimensions = 1,
                        }
                        match class {
                            naga::ImageClass::Sampled { kind: naga::ScalarKind::Uint , .. } => texture_group.kind = TextureKind::Uint,
                            naga::ImageClass::Sampled { kind: naga::ScalarKind::Float , .. } => texture_group.kind = TextureKind::Float,
                            _ => panic!("Many texture types not supported") 
                        }
                        self.uniform_descriptors.insert(variable.0, texture_group.image.as_mut().unwrap());
                    } else if let naga::TypeInner::Sampler { .. } = kind.inner {
                        texture_group.sampler = Some(uniform);
                        self.uniform_descriptors.insert(variable.0, texture_group.sampler.as_mut().unwrap());
                    }
                } else {
                    panic!("WILD!!!!!!");
                }
            } else {
                //get or insert Uniform descriptor or panic if its not a uniform descriptor
                let group: &mut GroupDescriptor = 
                    if matches!(group, Some(GroupDescriptor::Uniform(..))) {
                        group.unwrap()
                    } else if group.is_none() {
                        self.groups.push((
                            binding.group,
                            GroupDescriptor::Uniform(
                                UniformGroupDescriptor {
                                    index: binding.group,
                                    uniforms: Vec::new(),
                                }
                            )
                        ));
                        &mut self.groups.last_mut().unwrap().1
                    } else {
                        panic!("Assigning texture to uniform group")
                    };


                if let GroupDescriptor::Uniform(uniform_group) = group {
                    if uniform.size.is_none() {
                        panic!("Worlds WGSL: Don't know what to do with variable: {}", uniform.name);
                    }
                    uniform_group.uniforms.push(uniform);
                    self.uniform_descriptors.insert(variable.0, uniform_group.uniforms.last_mut().unwrap());
                } else {
                    panic!("WILD!!!!!!");
                }
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
