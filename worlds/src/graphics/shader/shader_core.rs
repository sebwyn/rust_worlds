use super::reflector::{reflect_shader, ReflectedShader};
use super::{UniformGroup, TextureGroup, descriptors::GroupDescriptor};

use crate::graphics::{RenderContext, Sampler};
use crate::graphics::Texture;

use std::rc::Rc;
use std::path::Path;
use std::borrow::Cow;
use std::collections::HashMap;
use std::mem::size_of;

pub struct ShaderDescriptor<'a> {
    pub file: &'a str,
}

//textures should always be bound together with a sampler so we can cheat
#[derive(Debug, Clone)]
pub struct UniformBinding {
    group: u32,
    binding: u32,
    size: u32,
}

#[derive(Debug, Clone)]
pub struct TextureBinding {
    group: u32,
}


pub enum Group {
    Texture(TextureGroup),
    Uniform(UniformGroup)
}

//because the number of uniforms is always relatively low, use vector instead of hash map
pub struct Shader {
    module: wgpu::ShaderModule,
    vs_entry_point: String,
    fs_entry_point: String,

    uniform_bindings: HashMap<String, UniformBinding>,
    texture_bindings: HashMap<String, TextureBinding>,

    groups: Vec<Group>,

    context: Rc<RenderContext>,
}

impl Shader {
    pub fn module(&self) -> &wgpu::ShaderModule { &self.module } 
    pub fn vs_entry_point(&self) -> &str { &self.vs_entry_point }
    pub fn fs_entry_point(&self) -> &str { &self.fs_entry_point }

    pub fn layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.groups.iter()
            .map(|group| {
                match group {
                    Group::Texture(texture_group) => &texture_group.layout,
                    Group::Uniform(uniform_group) => &uniform_group.layout,
                }
            })
            .collect()
    }

    pub fn bind_uniforms<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for (index, group) in self.groups.iter().enumerate() {
            let bind_group = match group {
                Group::Texture(texture_group) => texture_group.bind_group.as_ref().expect("Uh oh texture not set!"), 
                Group::Uniform(uniform_group) => &uniform_group.bind_group,
            };

            render_pass.set_bind_group(index as u32, bind_group, &[]);
        }
    }
}

impl Shader {
    //when created, it will create all of the cpu side resources for managing a shader
    //this means bind groups, bind group layouts, all of that
    pub fn new(descriptor: &ShaderDescriptor, context: Rc<RenderContext>) -> Self {
        //load our string into a cow
        let file = Path::new(descriptor.file);
        let shader_source = std::fs::read_to_string(file).unwrap_or_else(|_| {
            panic!("Can't read shader file: {}", descriptor.file)
        });

        let shader_name = file.file_name().unwrap().to_str().unwrap();

        //create our shader module 
        let module = 
            context.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_name),
                source: wgpu::ShaderSource::Wgsl(Cow::from(&shader_source)),
            });

        let ReflectedShader { 
            vs_entry_point, 
            fs_entry_point, 
            group_descriptors, 
        } = reflect_shader(&shader_source);


        let mut texture_bindings = HashMap::new();
        let mut uniform_bindings = HashMap::new();
        let mut groups = Vec::new();

        for group in group_descriptors.into_iter() {
            match group {
                GroupDescriptor::Uniform(uniform_group) => {
                    //add this groups uniforms to our uniforms
                    for uniform in uniform_group.uniforms.iter() {
                        uniform_bindings.insert(uniform.name.clone(), UniformBinding { 
                            group: uniform_group.index, 
                            binding: uniform.binding, 
                            size: uniform.size.unwrap()
                        });
                    }
                    groups.push(Group::Uniform(UniformGroup::new(&context.device, shader_name, uniform_group)));
                },
                GroupDescriptor::Texture(texture_group) => {
                    let index = texture_group.index;
                    let new_group = TextureGroup::new(&context.device, texture_group);
                    texture_bindings.insert(String::from(new_group.name()), TextureBinding { group: index });
                    
                    groups.push(Group::Texture(new_group));

                },
            }
        }


        Self {
            vs_entry_point,
            fs_entry_point,
            module,

            uniform_bindings,
            texture_bindings,
            groups,

            context,
        }
    }

    pub fn set_uniform<T>(&self, name: &str, value: T) -> Result<(), ()> 
    where
        T: bytemuck::Pod
    {
        let binding = self.uniform_bindings.get(name).cloned().ok_or(())?;

        let bind_group = self.groups.get(binding.group as usize).ok_or(())?;
        if let Group::Uniform(uniform_group) = bind_group {
            assert!(size_of::<T>() == binding.size as usize, 
                "Setting a uniform with a value not of the same size! {}", binding.size
            );

            let uniform_buffer = uniform_group.buffers.get(binding.binding as usize).ok_or(())?;
            self.context.queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[value]));

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn update_texture(&mut self, name: &str, texture: &Texture, sampler: Option<&Sampler>) -> Result<(), ()> 
    {
        let binding = self.texture_bindings.get(name).cloned().ok_or(())?;
        let texture_group = self.groups.get_mut(binding.group as usize).ok_or(())?;
        if let Group::Texture(texture_group) = texture_group {
            texture_group.update_texture(texture, sampler, &self.context.device);
            Ok(())
        } else {
            Err(())
        }
    }
}

