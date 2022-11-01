use bevy_ecs::prelude::*;

use crate::two_dimensional::sprite::Sprite;

#[derive(Component)]
pub struct Board {
    children: Vec<Vec<Entity>>,
}

#[derive(Component)]
pub struct Tile {
    piece: Option<Entity>,
}

impl Board {
    pub fn init(world: &mut World) {
        let mut children = Vec::new();
        for x in 0..8 {
            let mut column = Vec::new();
            for y in 0..8 {
                let color = if (x + y) % 2 == 0 {
                    [0.632, 0.3f32, 0f32]
                } else {
                    [1f32, 1f32, 1f32]
                };

                let position = [x as f32, y as f32];
                let dimensions = [1f32, 1f32];

                let tile = if y == 1 || y == 6 {
                    Tile {
                        piece: Some(
                            world
                                .spawn()
                                .insert(
                                    Sprite::new(position, dimensions, [1f32, 1f32, 1f32])
                                        .with_tile_in_texture("chess_piece_bitmap.png", 2, 6, 1, 0),
                                )
                                .id(),
                        ),
                    }
                } else {
                    Tile { piece: None }
                };

                let tile = world
                    .spawn()
                    .insert(Sprite::new(position, dimensions, color).with_depth(1))
                    .insert(tile)
                    .id();
                column.push(tile);
            }
            children.push(column);
        }

        world.spawn().insert(Self { children });
    }
}
