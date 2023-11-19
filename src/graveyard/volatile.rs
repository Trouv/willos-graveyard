//! Plugin providing the core logic for [`Volatile`] entities.
//!
//! Volatile entities are "Solid" initially.
//! Once they come into contact with another Volatile entity - they are both "Sublimated".
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    history::{FlushHistoryCommands, HistoryPlugin},
    utils::any_match_filter,
    GameState,
};

/// Plugin providing the core logic for [`Volatile`] entities.
///
/// Volatile entities are "Solid" initially.
/// Once they come into contact with another Volatile entity - they are both "Sublimated".
pub struct VolatilePlugin;

/// `SystemSet` performing sublimation [`Volatile`] entities.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Sublimation;

impl Plugin for VolatilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HistoryPlugin::<Volatile, _>::run_in_state(
            GameState::Graveyard,
        ))
        .add_systems(
            Update,
            sublimation
                .run_if(in_state(GameState::Graveyard))
                .run_if(any_match_filter::<(With<Volatile>, Changed<GridCoords>)>)
                .after(FlushHistoryCommands)
                .in_set(Sublimation),
        );
    }
}

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
    /// Returns `true` if this instance is [`Volatile::Solid`].
    pub fn is_solid(&self) -> bool {
        matches!(self, Volatile::Solid)
    }

    /// Sets this instance to [`Volatile::Sublimated`].
    pub fn sublimate(&mut self) {
        *self = Volatile::Sublimated;
    }
}

/// System performing sublimation logic.
///
/// Obtains a separate query for moving volatiles from all volatiles.
/// This is so it can split moving volatiles and stationary volatiles into separate collections.
/// This allows us to limit our collision detection to checking moved-volatiles against all-volatiles,
/// rather than all-volatiles against all-volatiles.
fn sublimation(
    moved_volatile_entities: Query<(), (With<Volatile>, Changed<GridCoords>)>,
    mut all_volatiles: Query<(Entity, &GridCoords, &mut Volatile)>,
) {
    // Split volatiles into moved and stationary collections.
    let (mut moved_volatiles, mut stationary_volatiles): (Vec<_>, Vec<_>) = all_volatiles
        .iter_mut()
        .partition(|(entity, ..)| moved_volatile_entities.contains(*entity));

    // Check for collisions between moved volatiles.
    for index in 0..moved_volatiles.len() - 1 {
        if let [(_, grid_coords_a, volatile_a), remaining_moved_volatiles @ ..] =
            &mut moved_volatiles[index..]
        {
            if volatile_a.is_solid() {
                for (_, grid_coords_b, volatile_b) in remaining_moved_volatiles.iter_mut() {
                    if volatile_b.is_solid() && grid_coords_a == grid_coords_b {
                        volatile_a.sublimate();
                        volatile_b.sublimate();
                    }
                }
            }
        }
    }

    // Check for collisions between moved volatiles and stationary volatiles.
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
