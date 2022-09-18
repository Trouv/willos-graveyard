pub mod actions;
pub mod text_button;

use crate::{
    previous_component::{PreviousComponentPlugin, TrackPreviousComponent},
    GameState,
};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, SystemLabel)]
pub enum UiLabels {
    Action,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PreviousComponentPlugin::<Interaction>::default())
            .add_event::<actions::UiAction>()
            .add_system(text_button::text_button_visuals.run_not_in_state(GameState::AssetLoading))
            .add_system(
                actions::ui_action
                    .run_not_in_state(GameState::AssetLoading)
                    .label(UiLabels::Action)
                    .after(TrackPreviousComponent),
            );

        #[cfg(feature = "ui-debug")]
        {
            app.add_enter_system(GameState::Gameplay, text_button::debug::debug_spawn_button)
                .add_system(actions::debug_print_action);
        }
    }
}
