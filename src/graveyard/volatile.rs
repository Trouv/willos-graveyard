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

fn sublimation(mut volatile_query: Query<(&GridCoords, &mut Volatile)>) {
    let mut combinations = volatile_query.iter_combinations_mut::<2>();
    while let Some([(grid_coords_a, mut volatile_a), (grid_coords_b, mut volatile_b)]) =
        combinations.fetch_next()
    {
        if grid_coords_a == grid_coords_b && volatile_a.is_solid() && volatile_b.is_solid() {
            volatile_a.sublimate();
            volatile_b.sublimate();
        }
    }
}
