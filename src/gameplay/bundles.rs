use crate::{
    gameplay::{components::*, xy_translation, Direction},
    SpriteHandles, UNIT_LENGTH,
};
use bevy::prelude::*;
use rand::seq::SliceRandom;

#[derive(Clone, Bundle)]
pub struct WallBundle {
    pub tile: Tile,
    pub rigid_body: RigidBody,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl WallBundle {
    pub fn new(coords: IVec2, sprite_handles: &SpriteHandles) -> WallBundle {
        let xy = xy_translation(coords);
        WallBundle {
            tile: Tile { coords },
            rigid_body: RigidBody::Static,
            sprite_bundle: SpriteBundle {
                material: sprite_handles.wall.clone_weak(),
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 1.),
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
        let xy = xy_translation(coords);
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
    history: History,
    rigid_body: RigidBody,
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
        let mut rng = rand::thread_rng();
        let xy = xy_translation(coords);
        InputBlockBundle {
            tile: Tile { coords },
            history: History { tiles: Vec::new() },
            rigid_body: RigidBody::Dynamic,
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
                    Direction::Up => sprite_handles
                        .w_block
                        .choose(&mut rng)
                        .unwrap()
                        .clone_weak(),
                    Direction::Left => sprite_handles
                        .a_block
                        .choose(&mut rng)
                        .unwrap()
                        .clone_weak(),
                    Direction::Down => sprite_handles
                        .s_block
                        .choose(&mut rng)
                        .unwrap()
                        .clone_weak(),
                    Direction::Right => sprite_handles
                        .d_block
                        .choose(&mut rng)
                        .unwrap()
                        .clone_weak(),
                },
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 1.),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Default, Bundle)]
pub struct GoalBundle {
    pub tile: Tile,
    pub goal: Goal,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl GoalBundle {
    pub fn new(coords: IVec2, sprite_handles: &SpriteHandles) -> GoalBundle {
        let xy = xy_translation(coords);
        GoalBundle {
            tile: Tile { coords },
            goal: Goal,
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
    pub history: History,
    pub rigid_body: RigidBody,
    pub player_state: PlayerState,
    pub timer: Timer,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl PlayerBundle {
    pub fn new(coords: IVec2, sprite_handles: &SpriteHandles) -> PlayerBundle {
        let xy = xy_translation(coords);
        PlayerBundle {
            tile: Tile { coords },
            history: History { tiles: Vec::new() },
            rigid_body: RigidBody::Dynamic,
            player_state: PlayerState::Waiting,
            timer: Timer::from_seconds(0.2, false),
            sprite_bundle: SpriteBundle {
                material: sprite_handles.player.clone_weak(),
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 2.),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Bundle)]
pub struct MoveTableBundle {
    pub tile: Tile,
    pub move_table: MoveTable,
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}

impl MoveTableBundle {
    pub fn new(player: Entity, coords: IVec2, sprite_handles: &SpriteHandles) -> MoveTableBundle {
        let xy = xy_translation(coords);
        MoveTableBundle {
            tile: Tile { coords },
            move_table: MoveTable {
                table: [[None; 4]; 4],
                player,
            },
            sprite_bundle: SpriteBundle {
                material: sprite_handles.player.clone_weak(),
                sprite: Sprite::new(Vec2::splat(UNIT_LENGTH)),
                transform: Transform::from_xyz(xy.x, xy.y, 0.),
                ..Default::default()
            },
        }
    }
}
