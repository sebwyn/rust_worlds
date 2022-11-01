use bevy_ecs::prelude::*;

use super::{SpriteVertex, TileView};

#[derive(Component, Debug)]
pub struct Sprite {
    pub position: [f32; 2],
    pub dimensions: [f32; 2],
    pub color: [f32; 3],

    pub depth: u32,

    texture_path: Option<String>,
    tile_view: Option<TileView>
}

impl Sprite {
    pub fn new(position: [f32; 2], dimensions: [f32; 2], color: [f32; 3]) -> Self {
        Self {
            position,
            dimensions,
            color,
            depth: 0,

            texture_path: None,
            tile_view: None
        }
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_texture(mut self, path: &str) -> Self {
        self.texture_path = Some(String::from(path));
        self
    }

    pub fn with_tile_in_texture(self, path: &str, rows: u32, cols: u32, row: u32, col: u32) -> Self {
        let mut texture_sprite = self.with_texture(path);
        texture_sprite.tile_view = Some(TileView::new(path, rows, cols, row, col));
        texture_sprite
    }

    pub fn texture_path(&self) -> &Option<String> {
        &self.texture_path
    }

    pub fn get_vertex_buffer(&self) -> Vec<SpriteVertex> {
        let tex_coords = if let Some(tile_view) = self.tile_view.as_ref() {
            tile_view.tex_coords()
        } else {
            //either the texture doesn't exist or there is one texture so we can just use default tex_coords;
            [[0f32, 0f32], [1f32, 0f32], [1f32, 1f32], [0f32, 1f32]]
        };

        let bl = SpriteVertex {
            position: self.position,
            tex_coord: tex_coords[0],
            color: self.color
        };

        let br = SpriteVertex {
            position: [self.position[0] + self.dimensions[0], self.position[1]],
            tex_coord: tex_coords[1],
            color: self.color
        };

        let tl = SpriteVertex {
            position: [self.position[0], self.position[1] + self.dimensions[1]],
            tex_coord: tex_coords[3],
            color: self.color
        };

        let tr = SpriteVertex {
            position: [self.position[0] + self.dimensions[0], self.position[1] + self.dimensions[1]],
            tex_coord: tex_coords[2],
            color: self.color
        };

        vec![bl, br, tl, tl, br, tr] 
    }
}