use std::collections::HashMap;

use bevy_ecs::prelude::*;
use image::Rgba;
use wgpu::{RenderPassDescriptor, RenderPipeline};

use crate::{graphics::{RenderContext, RenderPass, Subpass, Uniform, Texture, TextureBindLayout}, two_dimensional::{camera::{CameraMatrix, Camera}}};

use super::{sprite_vertex::SpriteVertex, Sprite};

use itertools::Itertools;
pub struct SpritePass {
    camera_uniform: Uniform,
    render_pipeline: RenderPipeline,
    texture_bind_layout: TextureBindLayout,
}

pub struct TextureCache(HashMap<String, (Texture, wgpu::BindGroup)>);

impl RenderPass for SpritePass {
    fn get_name() -> &'static str {
        "Sprite Pass"
    }

    fn get_init_system() -> Box<dyn bevy_ecs::system::System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::init))
    }

    fn get_render_system() -> Box<dyn bevy_ecs::system::System<In = (), Out = ()>> {
        Box::new(IntoSystem::into_system(Self::render))
    }
}

impl SpritePass {
    fn init(mut commands: Commands, render_context: Res<RenderContext>) {
    //create a camera uniform
    let camera_uniform = Uniform::new::<CameraMatrix>(render_context.as_ref(), 0);

    //create our texture layout here, and store it
    let texture_bind_layout = TextureBindLayout::new(0, 1, &render_context);

    let shader = render_context
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });

    let render_pipeline_layout =
        render_context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_uniform.bind_group_layout, texture_bind_layout.bind_group_layout()],
                push_constant_ranges: &[],
            });
    //create our pipeline here
    let render_pipeline =
        render_context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",           // 1.
                    buffers: &[SpriteVertex::desc()], // 2.
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: render_context.config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: None,                  //Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });

        let blank_texture = Texture::new::<Rgba<u8>>(10, 10, vec![255, 255, 255, 255], &render_context);
        let blank_texture_bind_group = texture_bind_layout.create_bind_group(&blank_texture, &render_context);
        
        let mut texture_cache = HashMap::new();
        texture_cache.insert(String::from(""), (blank_texture, blank_texture_bind_group));

        commands.insert_resource(Self { camera_uniform, render_pipeline, texture_bind_layout });
        commands.insert_resource(TextureCache(texture_cache));
    }

    fn render(
        sprites: Query<&Sprite>,
        cameras: Query<&Camera>,
        mut sprite_pass: ResMut<SpritePass>,
        mut texture_cache: ResMut<TextureCache>,
        mut subpass: ResMut<Subpass>,
        render_context: Res<RenderContext>,
    ) {

        let mut staging_render_sets: Vec<HashMap<String, (&wgpu::BindGroup, Vec<SpriteVertex>)>> = Vec::new();

        //construct our vertex buffers for each sprite with the same texture

        //construct a vertex buffer from sprites
        //may want to do this kind of caching in an update function, idk?
        for sprite in sprites.iter() { 
            let empty_string = String::from("");
            let sprite_texture = sprite.texture_path().as_ref().unwrap_or(&empty_string);
            if !texture_cache.0.contains_key(sprite_texture) {
                let texture = Texture::load(sprite_texture, &render_context);
                let bind_group = sprite_pass.texture_bind_layout.create_bind_group(&texture, &render_context);
                texture_cache.0.insert(sprite_texture.clone(), (texture, bind_group));
            }
        }

        //sort by depth here
        let depth_sorted_sprites = sprites.iter().sorted_by(|sprite_1, sprite_2| Ord::cmp(&sprite_2.depth, &sprite_1.depth)).group_by(|sprite| sprite.depth);
        /*for sprite in depth_sorted_sprites.into_iter() {
            println!("{} {:?}", sprite.0, sprite.1.collect::<Vec<&Sprite>>());
        }*/

        for (depth, sprites) in depth_sorted_sprites.into_iter() {
            let mut staging_buffers: HashMap<String, (&wgpu::BindGroup, Vec<SpriteVertex>)> = HashMap::new();
            let sprites: Vec<&Sprite> = sprites.collect();

            for sprite in sprites {
                let empty_string = String::from("");
                let sprite_texture = sprite.texture_path().as_ref().unwrap_or(&empty_string);
                if let Some((_, vertices)) = staging_buffers.get_mut(sprite_texture) {
                    let mut sprite_vertices = sprite.get_vertex_buffer();
                    vertices.append(&mut sprite_vertices);
                    continue;
                } else if let Some((_, bind_group)) = texture_cache.0.get(sprite_texture) {
                    staging_buffers.insert(sprite_texture.clone(), (bind_group, sprite.get_vertex_buffer()));
                    continue;
                }
            }

            staging_render_sets.push(staging_buffers);
        }

        /*for sprite in sprites.iter() {
            let empty_string = String::from("");
            let sprite_texture = sprite.texture_path().as_ref().unwrap_or(&empty_string);
            if let Some((_, vertices)) = staging_render_sets.get_mut(sprite_texture) {
                let mut sprite_vertices = sprite.get_vertex_buffer();
                vertices.append(&mut sprite_vertices);
                continue;
            } else if let Some((_, bind_group)) = texture_cache.0.get(sprite_texture) {
                staging_render_sets.insert(sprite_texture.clone(), (bind_group, sprite.get_vertex_buffer()));
                continue;
            }
        }*/

        let mut render_sets: Vec<Vec<(&wgpu::BindGroup, wgpu::Buffer, u32)>> = Vec::new();
        for staged_render_set in staging_render_sets.into_iter() {
            let mut depth_set = Vec::new();
            //generate our vertex buff
            for (_, (bind_group, vertex_vec)) in staged_render_set.into_iter() {
                let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
                    &render_context.device,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Sprite Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertex_vec),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );
                depth_set.push((bind_group, vertex_buffer, vertex_vec.len() as u32));
            }

            render_sets.push(depth_set);
        }

        let camera = cameras.get_single().expect("There should be a camera in the scene!");
        //update our camera uniform
        sprite_pass.camera_uniform.set_buffer(render_context.as_ref(), camera.get_matrix());

        let subpass = &mut *subpass;
        let encoder = subpass.encoder.as_mut().unwrap();

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Sprite Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &subpass.texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });


        render_pass.set_pipeline(&sprite_pass.render_pipeline);
        render_pass.set_bind_group(0, &sprite_pass.camera_uniform.bind_group, &[]);
        for depth_set in render_sets.iter() {
            for (texture_bind_group, vertex_buffer, num_vertices) in depth_set.iter() {
                render_pass.set_bind_group(1, texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..*num_vertices, 0..1);
            }
        }
    }
}
