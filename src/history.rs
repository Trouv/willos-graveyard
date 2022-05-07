pub use crate::gameplay::components::*;
pub use bevy::prelude::*;
pub use bevy_ecs_ldtk::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HistoryEvent {
    Record,
    Rewind,
    Reset,
}

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct History {
    pub tiles: Vec<GridCoords>,
}

pub fn rewind(
    mut player_query: Query<&mut PlayerState>,
    input: Res<Input<KeyCode>>,
    mut objects_query: Query<(&mut History, &mut GridCoords)>,
    mut history_event_writer: EventWriter<HistoryEvent>,
) {
    if let Ok(PlayerState::Waiting | PlayerState::Dead) = player_query.get_single() {
        if input.just_pressed(KeyCode::Z) {
            let mut rewind_happened = false;
            for (mut history, mut grid_coords) in objects_query.iter_mut() {
                if let Some(prev_state) = history.tiles.pop() {
                    *grid_coords = prev_state;
                    rewind_happened = true;
                }
            }

            if rewind_happened {
                *player_query.single_mut() = PlayerState::Waiting;
                history_event_writer.send(HistoryEvent::Rewind);
            }
        }
    }
}

pub fn reset(
    mut player_query: Query<&mut PlayerState>,
    input: Res<Input<KeyCode>>,
    mut objects_query: Query<(&mut History, &mut GridCoords)>,
    mut history_event_writer: EventWriter<HistoryEvent>,
) {
    if let Ok(PlayerState::Waiting | PlayerState::Dead) = player_query.get_single() {
        if input.just_pressed(KeyCode::R) {
            let mut reset_happened = false;
            for (mut history, mut grid_coords) in objects_query.iter_mut() {
                if let Some(initial_state) = history.tiles.get(0) {
                    *grid_coords = *initial_state;
                    reset_happened = true;
                    history.tiles = Vec::new();
                }
            }

            if reset_happened {
                *player_query.single_mut() = PlayerState::Waiting;
                history_event_writer.send(HistoryEvent::Reset);
            }
        }
    }
}
