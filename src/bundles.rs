use crate::{gameplay::components::*, history::History};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Clone, Bundle, LdtkIntCell)]
pub struct WallBundle {
    #[from_int_grid_cell]
    rigid_body: RigidBody,
}

#[derive(Clone, Bundle, LdtkEntity)]
pub struct InputBlockBundle {
    #[grid_coords]
    grid_coords: GridCoords,
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
    #[grid_coords]
    pub grid_coords: GridCoords,
    pub goal: Goal,
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
}

#[derive(Clone, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[grid_coords]
    pub grid_coords: GridCoords,
    pub history: History,
    #[from_entity_instance]
    pub rigid_body: RigidBody,
    pub player_state: PlayerState,
    pub movement_timer: MovementTimer,
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub player_animation_state: PlayerAnimationState,
}

#[derive(Clone, Bundle, LdtkEntity)]
pub struct MoveTableBundle {
    #[grid_coords]
    pub grid_coords: GridCoords,
    pub move_table: MoveTable,
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
}

#[derive(Clone, Bundle, LdtkEntity)]
pub struct GrassBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub wind_timer: WindTimer,
}

#[derive(Clone, Bundle, LdtkIntCell)]
pub struct ExorcismBlockBundle {
    pub exorcism_block: ExorcismBlock,
}
