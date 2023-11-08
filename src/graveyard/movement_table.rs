//! Plugin and components providing functionality for the movement table, which alters Willo's
//! abilities based off the placement of gravestones.
use crate::{
    from_component::FromComponentSet,
    graveyard::{
        gravestone::GraveId,
        volatile::Volatile,
        willo::{MovementTimer, WilloAnimationState, WilloSets, WilloState},
    },
    history::FlushHistoryCommands,
    sokoban::{Direction, SokobanCommands, SokobanSets},
    GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

/// Plugin providing functionality for the movement table, which alters Willo's abilities based off
/// the placement of gravestones.
pub struct MovementTablePlugin;

impl Plugin for MovementTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                movement_table_update
                    .run_if(in_state(GameState::Graveyard))
                    .before(WilloSets::Input),
                move_willo_by_table
                    .run_if(in_state(GameState::Graveyard))
                    .after(SokobanSets::LogicalMovement)
                    .after(FlushHistoryCommands)
                    .before(FromComponentSet),
            ),
        )
        .register_ldtk_entity::<MovementTableBundle>("Table");
    }
}

/// Defines the order that the four [Direction]s go in on the table's rank and file.
pub const DIRECTION_ORDER: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Down,
    Direction::Right,
];

/// Component that marks the movement table and stores the current placement of gravestones.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct MovementTable {
    /// 4x4 table marking the locations of gravestones, identified by the [GraveId] they are
    /// associated with
    pub table: [[Option<GraveId>; 4]; 4],
}

#[derive(Clone, Bundle, LdtkEntity)]
struct MovementTableBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    move_table: MovementTable,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

fn movement_table_update(
    mut table_query: Query<(&GridCoords, &mut MovementTable)>,
    input_block_query: Query<(&GridCoords, &GraveId, &Volatile)>,
) {
    for (table_grid_coords, mut table) in table_query.iter_mut() {
        let mut new_table = [[None; 4]; 4];
        for (input_grid_coords, input_block, volatile) in input_block_query.iter() {
            if volatile.is_solid() {
                let diff = *input_grid_coords - *table_grid_coords;
                let x_index = diff.x - 1;
                let y_index = -1 - diff.y;
                if (0..4).contains(&x_index) && (0..4).contains(&y_index) {
                    // key block is in table
                    new_table[y_index as usize][x_index as usize] = Some(*input_block);
                }
            }
        }

        if table.table != new_table {
            table.table = new_table;
        }
    }
}

fn move_willo_by_table(
    table_query: Query<&MovementTable>,
    mut willo_query: Query<(
        Entity,
        &mut MovementTimer,
        &mut WilloState,
        &mut WilloAnimationState,
    )>,
    mut sokoban_commands: SokobanCommands,
    time: Res<Time>,
) {
    for table in table_query.iter() {
        if let Ok((entity, mut timer, mut willo, mut willo_animation_state)) =
            willo_query.get_single_mut()
        {
            timer.0.tick(time.delta());

            if timer.0.finished() {
                match *willo {
                    WilloState::RankMove(key) => {
                        for (i, rank) in table.table.iter().enumerate() {
                            if rank.contains(&Some(key)) {
                                let direction = DIRECTION_ORDER[i];
                                sokoban_commands.move_block(entity, direction);
                                *willo_animation_state = WilloAnimationState::Idle(direction);
                            }
                        }
                        *willo = WilloState::FileMove(key);
                        timer.0.reset();
                    }
                    WilloState::FileMove(key) => {
                        for rank in table.table.iter() {
                            for (i, cell) in rank.iter().enumerate() {
                                if *cell == Some(key) {
                                    let direction = DIRECTION_ORDER[i];
                                    sokoban_commands.move_block(entity, direction);
                                    *willo_animation_state = WilloAnimationState::Idle(direction);
                                }
                            }
                        }
                        *willo = WilloState::Waiting;
                        timer.0.reset();
                    }
                    _ => {}
                }
            }
        }
    }
}
