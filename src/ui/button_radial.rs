//! Simple plugin providing functionality for displaying the state of a button's interaction.
use bevy::prelude::*;

use crate::GameState;

/// Simple plugin providing functionality for displaying the state of a button's interaction.
///
/// To use, buttons should have a child with a UiImage marked as [ButtonRadial].
pub struct ButtonRadialPlugin;

impl Plugin for ButtonRadialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            highlight_button_radial.run_if(not(in_state(GameState::AssetLoading))),
        );
    }
}

/// Marker component for the background highlight radial on a button.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct ButtonRadial;

/// System that alters the visuals of a button radial to show interaction.
fn highlight_button_radial(
    text_buttons: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut button_radials: Query<(&mut BackgroundColor, &Parent), With<ButtonRadial>>,
) {
    for (button_entity, interaction) in text_buttons.iter() {
        if let Some((mut radial_color, _)) = button_radials
            .iter_mut()
            .find(|(_, parent)| parent.get() == button_entity)
        {
            match interaction {
                Interaction::None => {
                    *radial_color = BackgroundColor(Color::NONE);
                }
                Interaction::Hovered => {
                    *radial_color = BackgroundColor(Color::WHITE);
                }
                Interaction::Pressed => {
                    *radial_color = BackgroundColor(Color::GRAY);
                }
            }
        }
    }
}
