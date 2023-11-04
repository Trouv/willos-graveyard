use bevy::prelude::*;

/// Component defining the volatility of an entity and its volatile state.
///
/// If two volatile solids share the same space, they both are sublimated.
/// What this means for a particular entity should be defined elsewhere.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Component)]
pub enum Volatile {
    #[default]
    Solid,
    Sublimated,
}
