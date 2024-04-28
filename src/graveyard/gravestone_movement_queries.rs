use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_ecs_ldtk::prelude::*;

use crate::graveyard::{arrow_block::MovementTile, gravestone::GraveId};

#[derive(Debug, SystemParam)]
pub struct GravestoneMovementQueries<'w, 's> {
    gravestones: Query<'w, 's, (&'static GridCoords, &'static GraveId)>,
    movement_tiles: Query<'w, 's, (&'static GridCoords, &'static MovementTile)>,
}

impl<'w, 's> GravestoneMovementQueries<'w, 's> {
    pub fn find_movement(&self, grave_id: &GraveId) -> Option<&MovementTile> {
        self.gravestones
            .iter()
            .find(|(_, this_grave_id)| &grave_id == this_grave_id)
            .and_then(|(grid_coords, _)| {
                self.movement_tiles
                    .iter()
                    .find(|(this_grid_coords, _)| &grid_coords == this_grid_coords)
            })
            .map(|(_, movement_tile)| movement_tile)
    }
}
