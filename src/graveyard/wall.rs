//! Plugin providing spawning logic for static walls such as bushes and fences.
use crate::graveyard::sokoban::SokobanBlock;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

/// Plugin providing spawning logic for static walls such as bushes and fences.
pub struct WallPlugin;

const WALL_INT_GRID_VALUES: &[i32] = &[1, 3, 4];

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        WALL_INT_GRID_VALUES.iter().for_each(|value| {
            app.register_ldtk_int_cell::<WallBundle>(*value);
        });
    }
}

#[derive(Clone, Bundle, LdtkIntCell)]
struct WallBundle {
    #[from_int_grid_cell]
    sokoban_block: SokobanBlock,
}

impl From<IntGridCell> for SokobanBlock {
    fn from(cell: IntGridCell) -> SokobanBlock {
        if WALL_INT_GRID_VALUES.contains(&cell.value) {
            SokobanBlock::Static
        } else {
            panic!("tried to give non-wall cell a SokobanBlock")
        }
    }
}
