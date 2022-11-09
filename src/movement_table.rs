use crate::{gameplay::components::*, *};
use bevy::prelude::*;

pub struct MovementTablePlugin;

impl Plugin for MovementTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            movement_table_update
                .run_in_state(GameState::Gameplay)
                .before(SystemLabels::Input),
        )
        .register_ldtk_entity::<MovementTableBundle>("Table");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct MovementTable {
    pub table: [[Option<KeyCode>; 4]; 4],
}

#[derive(Clone, Bundle, LdtkEntity)]
struct MovementTableBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    move_table: MovementTable,
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

fn movement_table_update(
    mut table_query: Query<(&GridCoords, &mut MovementTable)>,
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
