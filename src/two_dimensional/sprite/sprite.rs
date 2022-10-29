use bevy_ecs::prelude::*;

use super::SpriteVertex;

#[derive(Component)]
pub struct Sprite {
    pub position: [f32; 2],
    pub dimensions: [f32; 2],
    pub color: [f32; 3]
}

impl Sprite {
    pub fn get_vertex_buffer(&self) -> Vec<SpriteVertex> {
        
        let bl = SpriteVertex {
            position: self.position,
            color: self.color
        };

        let br = SpriteVertex {
            position: [self.position[0] + self.dimensions[0], self.position[1]],
            color: self.color
        };

        let tl = SpriteVertex {
            position: [self.position[0], self.position[1] + self.dimensions[1]],
            color: self.color
        };

        let tr = SpriteVertex {
            position: [self.position[0] + self.dimensions[0], self.position[1] + self.dimensions[1]],
            color: self.color
        };

        vec![bl, br, tl, tl, br, tr] 
    }
}