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
