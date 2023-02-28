//! Plugin, components, systems, and events related to common UI patterns.

pub mod actions;
pub mod font_scale;
pub mod text_button;

use crate::GameState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

/// Plugin providing functionality for common UI patterns.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(font_scale::FontScalePlugin)
            .add_system(text_button::text_button_visuals.run_not_in_state(GameState::AssetLoading));
    }
}
