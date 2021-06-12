use crate::{gameplay::components::*, SpriteHandles, UNIT_LENGTH};
use bevy::prelude::*;

#[derive(Clone, Default, Bundle)]
pub struct WallBundle {
    pub tile: Tile,
    pub blocker: Blocker,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl WallBundle {
    fn new(coords: IVec2, sprite_handles: &SpriteHandles) -> WallBundle {
        let xy_translation = Vec2::new(coords.x as f32, coords.y as f32) * UNIT_LENGTH;
        WallBundle {
            tile: Tile { coords },
            blocker: Blocker,
            sprite_bundle: SpriteBundle {
                material: sprite_handles.wall.clone_weak(),
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy_translation.x, xy_translation.y, 0.),
                ..Default::default()
            },
        }
    }
}
