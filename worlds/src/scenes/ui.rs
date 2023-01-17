use std::rc::Rc;

use rayon::prelude::*;
use winit::event::VirtualKeyCode;

use crate::ui::{UiRenderer, Layout};
use crate::{graphics::RenderApi, core::Scene};

use crate::core::{Event, Window};

pub struct Ui {
    ui: UiRenderer,
    text: String,
}

impl Ui {
    fn get_voxel_color(index: u32) -> Layout {
        let x = index % 16;
        let y = index / 16;
        Layout::new((1.0 / 16.0) * x as f32, (1.0 / 16.0) * y as f32, (1.0 / 16.0), (1.0 / 16.0))
    }
}

//allows for swapping of scenes, potentially hotswapping!!!
impl Scene for Ui {
    fn new(_window: Rc<Window>, api: &RenderApi) -> Self {
        //load our color palette
        let palette_file = std::fs::read_to_string("resources/palette.txt").unwrap();
        let colors: Vec<u32> = palette_file.lines().map(|color| {
            let color = u32::from_str_radix(color, 16).unwrap();
            (color << 8) | 0xFF
        }).collect();

        //generate a palette texture here, a 16 by 16 2d texture, with ten pixels per color, so 160, by 160
        //generate our voxels, create a texture, and set the texture uniform on the pipeline (no live updating right now)
        let voxels: Vec<u8> = (0..(160)*(160)).into_par_iter().flat_map(|i| {
            let x = (i % 160) / 10;
            let y = i / 1600;

            let pos = (y * x) + x;

            let color = colors[pos];
            [((color & 0xFF000000) >> 24) as u8, ((color & 0xFF0000) >> 16) as u8, ((color & 0xFF00) >> 8) as u8, (color & 0xFF) as u8]
        }).collect();

        //create the texture here
        let texture = api.create_texture::<u32>((160, 160, 1), wgpu::TextureFormat::Rgba8UnormSrgb);
        texture.write_buffer(&voxels);
        let sampler = api.create_sampler();

        let mut ui = UiRenderer::new(api);
        ui.set_sprite_map(&texture, Some(&sampler));
        
        Ui { 
            ui,
            text: "Hello World".to_string(),
        }


    }

    fn update(&mut self, events: &[Event]) {
        for event in events.iter() {        
            if let Event::KeyPressed(key, _) = event {
                
                if let Some(c) = event.get_character() {
                    self.text.push(c);
                }

                if *key == VirtualKeyCode::Tab {
                    self.text += "    ";
                } else if *key == VirtualKeyCode::Back {
                    self.text.pop();
                }
            }
        }

        let box_layout = Layout::new(10.0, 590.0, 400.0, 580.0);

        // self.ui.put_text(&self.text, 20, 580, 380);
        // self.ui.put_rect(10, 590, 400, 580, [0.022, 0.025, 0.034, 1.0]);
        self.ui.put_text_box("Looking at:", [0.022, 0.025, 0.034, 1.0], &box_layout);

        let palette_layout = Layout::new(120.0, 400.0, 160.0, 160.0);
        let texture_layout = Self::get_voxel_color(12);

        //render our palette texture in its entirety
        self.ui.put_image(&palette_layout, &texture_layout);

        self.ui.update(events);
    }

    fn render(&mut self, surface_view: &wgpu::TextureView, render_api: &RenderApi) {       
        let mut encoder = render_api.begin_render();

        self.ui.render(surface_view, &mut encoder);

        render_api.end_render(encoder);
    }
}