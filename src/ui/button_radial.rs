use bevy::prelude::*;

pub struct ButtonRadialPlugin;

impl Plugin for ButtonRadialPlugin {
    fn build(&self, app: &mut App) {}
}

/// Marker component for the background highlight radial on "text button"s
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct ButtonRadial;
