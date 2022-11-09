use crate::gameplay::components::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

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
