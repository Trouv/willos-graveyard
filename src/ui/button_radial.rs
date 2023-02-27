use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::GameState;

pub struct ButtonRadialPlugin;

impl Plugin for ButtonRadialPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(highlight_button_radial.run_not_in_state(GameState::AssetLoading));
    }
}

/// Marker component for the background highlight radial on "text button"s
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct ButtonRadial;

/// System that alters the visuals of a button radial to show interaction
pub fn highlight_button_radial(
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
                Interaction::Clicked => {
                    *radial_color = BackgroundColor(Color::GRAY);
                }
            }
        }
    }
}
