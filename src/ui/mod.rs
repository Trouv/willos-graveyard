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
        app.add_plugin(font_scale::FontScalePlugin)
            .add_plugin(button_radial::ButtonRadialPlugin)
            .add_plugin(UiAtlasImagePlugin)
            .add_plugin(icon_button::IconButtonPlugin);
    }
}
