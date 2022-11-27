//! Plugin and components providing functionality for the movement table, which alters Willo's
//! abilities based off the placement of gravestones.
use crate::{
    gravestone::Gravestone,
    history::FlushHistoryCommands,
    sokoban::SokobanLabels,
    willo::{MovementTimer, WilloAction, WilloLabels, WilloMovementEvent, WilloState},
    GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

/// Plugin providing functionality for the movement table, which alters Willo's abilities based off
/// the placement of gravestones.
pub struct MovementTablePlugin;

impl Plugin for MovementTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            movement_table_update
                .run_in_state(GameState::Gameplay)
                .before(WilloLabels::Input),
        )
        .add_system(
            move_willo_by_table
                .run_in_state(GameState::Gameplay)
                .after(SokobanLabels::GridCoordsMovement)
                .after(FlushHistoryCommands),
        )
        .register_ldtk_entity::<MovementTableBundle>("Table");
    }
}

/// Enumerates the four directions that are exposed on the movement table.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

/// Defines the order that the four [Direction]s go in on the table's rank and file.
pub const DIRECTION_ORDER: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Down,
    Direction::Right,
];

impl From<Direction> for IVec2 {
    fn from(direction: Direction) -> IVec2 {
        match direction {
            Direction::Up => IVec2::Y,
            Direction::Left => IVec2::new(-1, 0),
            Direction::Down => IVec2::new(0, -1),
            Direction::Right => IVec2::X,
        }
    }
}

/// Component that marks the movement table and stores the current placement of gravestones.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct MovementTable {
    /// 4x4 table marking the locations of gravestones, identified by the [WilloAction] they are
    /// associated with
    pub table: [[Option<WilloAction>; 4]; 4],
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
    input_block_query: Query<(&GridCoords, &Gravestone)>,
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

fn move_willo_by_table(
    table_query: Query<&MovementTable>,
    mut willo_query: Query<(&mut MovementTimer, &mut WilloState)>,
    mut movement_writer: EventWriter<WilloMovementEvent>,
    time: Res<Time>,
) {
    for table in table_query.iter() {
        if let Ok((mut timer, mut willo)) = willo_query.get_single_mut() {
            timer.0.tick(time.delta());

            if timer.0.finished() {
                match *willo {
                    WilloState::RankMove(key) => {
                        for (i, rank) in table.table.iter().enumerate() {
                            if rank.contains(&Some(key)) {
                                movement_writer.send(WilloMovementEvent {
                                    direction: DIRECTION_ORDER[i],
                                });
                            }
                        }
                        *willo = WilloState::FileMove(key);
                        timer.0.reset();
                    }
                    WilloState::FileMove(key) => {
                        for rank in table.table.iter() {
                            for (i, cell) in rank.iter().enumerate() {
                                if *cell == Some(key) {
                                    movement_writer.send(WilloMovementEvent {
                                        direction: DIRECTION_ORDER[i],
                                    });
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
