//! Plugin, components, systems, and events related to common UI patterns.

pub mod action;
pub mod button_prompt;
pub mod button_radial;
pub mod font_scale;
pub mod icon_button;
pub mod text_button;

use crate::ui_atlas_image::UiAtlasImagePlugin;
use bevy::prelude::*;

/// Plugin providing functionality for common UI patterns.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            font_scale::FontScalePlugin,
            button_radial::ButtonRadialPlugin,
            UiAtlasImagePlugin,
            icon_button::IconButtonPlugin,
        ));
    }
}
