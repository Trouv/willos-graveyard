//! Plugin providing logic for all graveyard entities and the entire graveyard state.
//!
//! So, the logic for core gameplay lives here.

mod control_display;
mod exorcism;
mod goal;
mod gravestone;
mod movement_table;
mod sokoban;
pub mod willo;
mod wind;

use bevy::prelude::*;

/// Plugin providing logic for all graveyard entities and the entire graveyard state.
///
/// So, the logic for core gameplay lives here.
pub struct GraveyardPlugin;

impl Plugin for GraveyardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(control_display::ControlDisplayPlugin)
            .add_plugin(willo::WilloPlugin)
            .add_plugin(sokoban::SokobanPlugin)
            .add_plugin(movement_table::MovementTablePlugin)
            .add_plugin(gravestone::GravestonePlugin)
            .add_plugin(goal::GoalPlugin)
            .add_plugin(exorcism::ExorcismPlugin)
            .add_plugin(wind::WindPlugin);
    }
}
