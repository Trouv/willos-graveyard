//! TODO: provide crate documentation after writing README.
// these two lints are triggered by normal system code a lot
#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(missing_docs)]

pub mod animation;
pub mod camera;
pub mod event_scheduler;
pub mod from_component;
pub mod graveyard;
pub mod history;
pub mod level_select;
pub mod level_transition;
pub mod nine_slice;
pub mod previous_component;
pub mod sokoban;
pub mod ui;
pub mod ui_atlas_image;
pub mod utils;

use animation::SpriteSheetAnimationPlugin;
use bevy::prelude::*;

use bevy_asset_loader::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_ecs_ldtk::prelude::*;

/// Length of the sides of tiles on the game-grid in bevy's coordinate space.
pub const UNIT_LENGTH: i32 = 32;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

/// All possible bevy states that the game can be in.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, States)]
pub enum GameState {
    #[default]
    /// Initial state of the game that perpares assets with `bevy_asset_loader`.
    AssetLoading,
    /// State that facilitates level transitions, see [level_transition].
    LevelTransition,
    /// State for the core gameplay that takes place on graveyards, see [graveyard].
    Graveyard,
    /// State for the level select menu, see [level_select].
    LevelSelect,
}

fn main() {
    let level_selection = if std::env::args().count() > 1 {
        let level_arg = std::env::args().next_back().unwrap();

        match level_arg.parse::<usize>() {
            Ok(num) => LevelSelection::index(num - 1),
            _ => LevelSelection::Identifier(level_arg),
        }
    } else {
        LevelSelection::index(0)
    };

    let mut app = App::new();

    app.insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EasingsPlugin,
            LdtkPlugin,
        ))
        .insert_resource(LdtkSettings {
            set_clear_color: SetClearColor::FromEditorBackground,
            ..default()
        })
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::LevelTransition)
                .load_collection::<AssetHolder>()
                .load_collection::<graveyard::gravestone::GravestoneAssets>()
                .load_collection::<graveyard::control_display::ControlDisplayAssets>()
                .load_collection::<ui::icon_button::IconButtonAssets>()
                .load_collection::<ui::button_prompt::ButtonPromptAssets>(),
        )
        .add_plugins((
            graveyard::GraveyardPlugin,
            SpriteSheetAnimationPlugin,
            ui::UiPlugin,
            level_select::LevelSelectPlugin,
            camera::CameraPlugin,
            level_transition::LevelTransitionPlugin,
        ))
        .insert_resource(level_selection.clone())
        .insert_resource(level_transition::TransitionTo(level_selection));

    #[cfg(feature = "inspector")]
    {
        app.add_plugins(WorldInspectorPlugin::new());
    }

    app.run()
}

/// Asset collection loaded during the `GameState::AssetLoading` state.
///
/// Each field provides a handle for a different core asset of the game.
#[derive(Debug, Default, AssetCollection, Resource)]
pub struct AssetHolder {
    /// Handle for all the LDtk info (level design).
    #[asset(path = "levels/willos-graveyard.ldtk")]
    pub ldtk: Handle<LdtkProject>,
    /// Handle for the game's spooky font.
    #[asset(path = "fonts/WayfarersToyBoxRegular-gxxER.ttf")]
    pub font: Handle<Font>,
    /// Handle for the image used to underline text on text buttons.
    #[asset(path = "textures/button-underline.png")]
    pub button_underline: Handle<Image>,
    /// Handle for the image used to highlight buttons on hover.
    #[asset(path = "textures/button-radial.png")]
    pub button_radial: Handle<Image>,
    /// Handle for the sound that plays on level completion.
    #[asset(path = "sfx/victory.wav")]
    pub victory_sound: Handle<AudioSource>,
    /// Handle for the sound that plays when Willo pushes a gravestone.
    #[asset(path = "sfx/push.wav")]
    pub push_sound: Handle<AudioSource>,
    /// Handle for the sound that plays when the player hits undo/reset.
    #[asset(path = "sfx/undo.wav")]
    pub undo_sound: Handle<AudioSource>,
    /// Handle for the tarot-card-inspired 9-slice image.
    #[asset(path = "textures/tarot.png")]
    pub tarot_sheet: Handle<Image>,
}
