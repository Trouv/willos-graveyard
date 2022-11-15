use crate::{gameplay::components::*, gameplay::DeathEvent, willo::WilloState};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub fn check_death(
    mut willo_query: Query<(Entity, &GridCoords, &mut WilloState)>,
    exorcism_query: Query<(Entity, &GridCoords), With<ExorcismBlock>>,
    mut death_event_writer: EventWriter<DeathEvent>,
) {
    if let Ok((entity, coords, mut willo)) = willo_query.get_single_mut() {
        if *willo != WilloState::Dead && exorcism_query.iter().any(|(_, g)| *g == *coords) {
            *willo = WilloState::Dead;
            death_event_writer.send(DeathEvent {
                willo_entity: entity,
            });
        }
    }
}
