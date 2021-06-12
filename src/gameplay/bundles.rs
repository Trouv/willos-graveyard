use crate::{
    gameplay::{components::*, xy_translation, Direction},
    SpriteHandles, UNIT_LENGTH,
};
use bevy::prelude::*;

#[derive(Clone, Default, Bundle)]
pub struct WallBundle {
    pub tile: Tile,
    pub blocker: Blocker,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl WallBundle {
    pub fn new(coords: IVec2, sprite_handles: &SpriteHandles) -> WallBundle {
        let xy = xy_translation(coords) * UNIT_LENGTH;
        WallBundle {
            tile: Tile { coords },
            blocker: Blocker,
            sprite_bundle: SpriteBundle {
                material: sprite_handles.wall.clone_weak(),
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 0.),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Default, Bundle)]
pub struct DirectionTileBundle {
    tile: Tile,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl DirectionTileBundle {
    pub fn new(
        direction: Direction,
        coords: IVec2,
        sprite_handles: &SpriteHandles,
    ) -> DirectionTileBundle {
        let xy = xy_translation(coords) * UNIT_LENGTH;
        DirectionTileBundle {
            tile: Tile { coords },
            sprite_bundle: SpriteBundle {
                material: match direction {
                    Direction::Up => sprite_handles.up.clone_weak(),
                    Direction::Left => sprite_handles.left.clone_weak(),
                    Direction::Right => sprite_handles.right.clone_weak(),
                    Direction::Down => sprite_handles.down.clone_weak(),
                },
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 0.),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Bundle)]
pub struct InputBlockBundle {
    tile: Tile,
    blocker: Blocker,
    pushable: Pushable,
    input_block: InputBlock,
    #[bundle]
    sprite_bundle: SpriteBundle,
}

impl InputBlockBundle {
    pub fn new(
        direction: Direction,
        coords: IVec2,
        sprite_handles: &SpriteHandles,
    ) -> InputBlockBundle {
        let xy = xy_translation(coords) * UNIT_LENGTH;
        InputBlockBundle {
            tile: Tile { coords },
            blocker: Blocker,
            pushable: Pushable,
            input_block: InputBlock {
                key_code: match direction {
                    Direction::Up => KeyCode::W,
                    Direction::Left => KeyCode::A,
                    Direction::Down => KeyCode::S,
                    Direction::Right => KeyCode::D,
                },
            },
            sprite_bundle: SpriteBundle {
                material: match direction {
                    Direction::Up => sprite_handles.w_block.clone_weak(),
                    Direction::Left => sprite_handles.a_block.clone_weak(),
                    Direction::Down => sprite_handles.s_block.clone_weak(),
                    Direction::Right => sprite_handles.d_block.clone_weak(),
                },
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 0.),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Default, Bundle)]
pub struct GoalBundle {
    pub tile: Tile,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl GoalBundle {
    pub fn new(coords: IVec2, sprite_handles: &SpriteHandles) -> GoalBundle {
        let xy = xy_translation(coords) * UNIT_LENGTH;
        GoalBundle {
            tile: Tile { coords },
            sprite_bundle: SpriteBundle {
                material: sprite_handles.goal.clone_weak(),
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 0.),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Bundle)]
pub struct PlayerBundle {
    pub tile: Tile,
    pub blocker: Blocker,
    pub pushable: Pushable,
    pub player_state: PlayerState,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl PlayerBundle {
    pub fn new(coords: IVec2, sprite_handles: &SpriteHandles) -> PlayerBundle {
        let xy = xy_translation(coords) * UNIT_LENGTH;
        PlayerBundle {
            tile: Tile { coords },
            blocker: Blocker,
            pushable: Pushable,
            player_state: PlayerState::Waiting,
            sprite_bundle: SpriteBundle {
                material: sprite_handles.player.clone_weak(),
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 0.),
                ..Default::default()
            },
        }
    }
}
