use super::reflector::{reflect_shader, ReflectedShader};
use super::{UniformGroup, TextureGroup};

use crate::graphics::RenderContext;
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

//potentially kind of slow, but this will represent a shader on the cpu side
pub struct Shader {
    module: wgpu::ShaderModule,
    vs_entry_point: String,
    fs_entry_point: String,

    uniform_bindings: HashMap<String, UniformBinding>,
    uniforms: Vec<UniformGroup>,

    texture_bindings: HashMap<String, TextureBinding>,
    textures: Vec<TextureGroup>,

    context: Rc<RenderContext>,
}

impl Shader {
    pub fn module(&self) -> &wgpu::ShaderModule { &self.module } 
    pub fn vs_entry_point(&self) -> &str { &self.vs_entry_point }
    pub fn fs_entry_point(&self) -> &str { &self.fs_entry_point }

    pub fn layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.uniforms.iter().map(|group| &group.layout).chain(self.textures.iter().map(|group| &group.layout)).collect()
    }

    pub fn bind_uniforms<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for (index, group) in self.uniforms.iter().enumerate() {
            render_pass.set_bind_group(index as u32, &group.bind_group, &[]);
        }

        /*for (index, group) in self.uniforms.iter().enumerate() {
            render_pass.set_bind_group(index as u32, &group.bind_group, &[]);
        }*/
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
            context.device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_name),
                source: wgpu::ShaderSource::Wgsl(Cow::from(&shader_source)),
            });

        let ReflectedShader { 
            vs_entry_point, 
            fs_entry_point, 
            mut texture_group_descriptors, 
            mut uniform_group_descriptors 
        } = reflect_shader(&shader_source);

        uniform_group_descriptors.sort_by_key(|group| group.index);

        let mut uniform_bindings = HashMap::new();
        let mut uniforms = Vec::new();

        uniform_group_descriptors.sort_by_key(|group| group.index);
        for group in uniform_group_descriptors.into_iter() {
            //add this groups uniforms to our uniforms
            for uniform in group.uniforms.iter() {
                uniform_bindings.insert(uniform.name.clone(), UniformBinding { 
                    group: group.index, 
                    binding: uniform.binding, 
                    size: uniform.size.unwrap()
                });
            }
            
            uniforms.push(UniformGroup::new(context.device(), shader_name, group));
        }


        let mut texture_bindings = HashMap::new();
        let mut textures = Vec::new();

        texture_group_descriptors.sort_by_key(|group| group.index);
        for group in texture_group_descriptors.into_iter() {
            let index = group.index;

            textures.push(TextureGroup::new(context.device(), group));
            let tex = textures.last().unwrap();

            texture_bindings.insert(String::from(tex.name()), TextureBinding { group: index });
        }

        Self {
            vs_entry_point,
            fs_entry_point,
            module,

            uniform_bindings,
            uniforms,
            
            texture_bindings,
            textures,

            context,
        }
    }
    

    pub fn get_uniform_binding(&self, name: &str) -> Option<UniformBinding> {
        self.uniform_bindings.get(name).cloned()
    }

    pub fn set_uniform<T>(&self, binding: &UniformBinding, value: T) -> Result<(), ()> 
    where
        T: bytemuck::Pod
    {
        let bind_group = self.uniforms.get(binding.group as usize).ok_or(())?;

        assert!(size_of::<T>() == binding.size as usize, 
            "Setting a uniform with a value not of the same size!"
        );

        let uniform_buffer = bind_group.buffers.get(binding.binding as usize).ok_or(())?;
        self.context.queue().write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[value]));

        Ok(())
    }

    pub fn get_texture_binding(&self, name: &str) -> Option<TextureBinding> {
        self.texture_bindings.get(name).cloned()
    }

    pub fn update_texture(&mut self, binding: &TextureBinding, texture: &Texture) -> Result<(), ()> 
    {
        let texture_group = self.textures.get_mut(binding.group as usize).ok_or(())?;
        texture_group.update_texture(texture, self.context.device());
        Ok(())
    }
}

