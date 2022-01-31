use crate::{
    gameplay::{components::*, xy_translation},
    SpriteHandles,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct TableBundle {
    #[ldtk_entity]
    tile: Tile,
    #[sprite_bundle]
    #[bundle]
    sprite_bundle: SpriteBundle,
}

#[derive(Clone, Bundle, LdtkEntity)]
pub struct InputBlockBundle {
    #[ldtk_entity]
    tile: Tile,
    history: History,
    #[from_entity_instance]
    rigid_body: RigidBody,
    #[from_entity_instance]
    input_block: InputBlock,
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    #[ldtk_entity]
    pub tile: Tile,
    pub goal: Goal,
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
}

#[derive(Clone, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[ldtk_entity]
    pub tile: Tile,
    pub history: History,
    #[from_entity_instance]
    pub rigid_body: RigidBody,
    pub player_state: PlayerState,
    pub timer: Timer,
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
}

#[derive(Clone, Bundle, LdtkEntity)]
pub struct MoveTableBundle {
    #[ldtk_entity]
    pub tile: Tile,
    pub move_table: MoveTable,
    #[sprite_bundle]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
}
