pub mod text_button;

use crate::GameState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(text_button::text_button_visuals.run_not_in_state(GameState::AssetLoading));

        #[cfg(feature = "ui-debug")]
        app.add_enter_system(GameState::Gameplay, text_button::debug::debug_spawn_button);
    }
}
