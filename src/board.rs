use bevy_ecs::prelude::*;

use crate::two_dimensional::sprite::Sprite;

#[derive(Component)]
pub struct Board {
    children: Vec<Vec<Entity>>,
}

#[derive(Component)]
pub struct Piece {
    entity: Entity,
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

                let tile = world
                    .spawn()
                    .insert(Sprite::new(position, dimensions, color).with_depth(1))
                    .id();
                column.push(tile);
            }
            children.push(column);
        }

        let board_data = [
            (1, 1),
            (1, 2),
            (1, 3),
            (1, 5),
            (1, 4),
            (1, 3),
            (1, 2),
            (1, 1),
        ];

        for (x, (row, col)) in board_data.iter().enumerate() {
            let position = [x as f32, 0 as f32];
            let dimensions = [1f32, 1f32];

            let piece =
                Piece {
                    entity: world
                        .spawn()
                        .insert(
                            Sprite::new(position, dimensions, [1f32, 1f32, 1f32])
                                .with_tile_in_texture("chess_piece_bitmap.png", 2, 6, *row, *col),
                        )
                        .id(),
                };

            world.get_entity_mut(children[0][x]).unwrap().insert(piece);
        }

        world.spawn().insert(Self { children });
    }
}
