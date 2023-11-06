use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

/// Component defining the volatility of an entity and its volatile state.
///
/// If two volatile solids share the same [`GridCoords`] space, they both are sublimated.
/// What this means for a particular entity should be defined elsewhere.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Component)]
pub enum Volatile {
    /// The entity is still a volatile solid.
    #[default]
    Solid,
    /// The entity has collided with another volatile solid, and has been subliminated.
    Sublimated,
}

impl Volatile {
    fn is_solid(&self) -> bool {
        matches!(self, Volatile::Solid)
    }

    fn sublimate(&mut self) {
        *self = Volatile::Sublimated;
    }
}

fn sublimation(
    moved_volatile_entities: Query<Entity, (With<Volatile>, Changed<GridCoords>)>,
    mut all_volatiles: Query<(Entity, &GridCoords, &mut Volatile), Changed<GridCoords>>,
) {
    let (mut moved_volatiles, mut stationary_volatiles): (Vec<_>, Vec<_>) = all_volatiles
        .iter_mut()
        .partition(|(entity, ..)| moved_volatile_entities.contains(*entity));

    for index in 1..moved_volatiles.len() {
        let (head_moved_volatiles, remaining_moved_volatiles) = moved_volatiles.split_at_mut(index);

        let (_, grid_coords_a, volatile_a) = head_moved_volatiles.last_mut().expect("TODO");

        if volatile_a.is_solid() {
            for (_, grid_coords_b, volatile_b) in remaining_moved_volatiles.iter_mut() {
                if volatile_b.is_solid() && grid_coords_a == grid_coords_b {
                    volatile_a.sublimate();
                    volatile_b.sublimate();
                }
            }
        }
    }

    for (_, grid_coords_a, volatile_a) in moved_volatiles.iter_mut() {
        if volatile_a.is_solid() {
            for (_, grid_coords_b, volatile_b) in stationary_volatiles.iter_mut() {
                if volatile_b.is_solid() && grid_coords_a == grid_coords_b {
                    volatile_a.sublimate();
                    volatile_b.sublimate();
                }
            }
        }
    }
}
