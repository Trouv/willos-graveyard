use crate::graveyard::sokoban::SokobanBlock;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell::<WallBundle>(1)
            .register_ldtk_int_cell::<WallBundle>(3)
            .register_ldtk_int_cell::<WallBundle>(4);
    }
}

#[derive(Clone, Bundle, LdtkIntCell)]
struct WallBundle {
    #[from_int_grid_cell]
    sokoban_block: SokobanBlock,
}

impl From<EntityInstance> for SokobanBlock {
    fn from(_: EntityInstance) -> SokobanBlock {
        SokobanBlock::Dynamic
    }
}

impl From<IntGridCell> for SokobanBlock {
    fn from(_: IntGridCell) -> SokobanBlock {
        SokobanBlock::Static
    }
}
