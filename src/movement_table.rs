use crate::{gameplay::components::*, *};
use bevy::prelude::*;

pub struct MovementTablePlugin;

impl Plugin for MovementTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            move_table_update
                .run_in_state(GameState::Gameplay)
                .before(SystemLabels::Input),
        )
        .register_ldtk_entity::<MoveTableBundle>("Table");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct MoveTable {
    pub table: [[Option<KeyCode>; 4]; 4],
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

fn move_table_update(
    mut table_query: Query<(&GridCoords, &mut MoveTable)>,
    input_block_query: Query<(&GridCoords, &InputBlock)>,
) {
    for (table_grid_coords, mut table) in table_query.iter_mut() {
        table.table = [[None; 4]; 4];
        for (input_grid_coords, input_block) in input_block_query.iter() {
            let diff = *input_grid_coords - *table_grid_coords;
            let x_index = diff.x - 1;
            let y_index = -1 - diff.y;
            if (0..4).contains(&x_index) && (0..4).contains(&y_index) {
                // key block is in table
                table.table[y_index as usize][x_index as usize] = Some(input_block.key_code);
            }
        }
    }
}
