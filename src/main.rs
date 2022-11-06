// these two lints are triggered by normal system code a lot
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod animation;
mod bundles;
mod event_scheduler;
mod from_component;
mod gameplay;
mod history;
mod level_select;
mod nine_slice;
mod previous_component;
mod resources;
mod sokoban;
mod sugar;
mod ui;
mod willo;

use animation::{FromComponentAnimator, SpriteSheetAnimationPlugin};
use bevy::{prelude::*, render::texture::ImageSettings};

use bevy_asset_loader::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;
use rand::Rng;

pub const UNIT_LENGTH: f32 = 32.;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, SystemLabel)]
pub enum SystemLabels {
    LoadAssets,
    Input,
    MoveTableUpdate,
    CheckDeath,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    AssetLoading,
    LevelTransition,
    Gameplay,
    LevelSelect,
}

fn main() {
    let level_selection = if std::env::args().count() > 1 {
        let level_arg = std::env::args().last().unwrap();

        match level_arg.parse::<usize>() {
            Ok(num) => LevelSelection::Index(num - 1),
            _ => LevelSelection::Identifier(level_arg),
        }
    } else {
        LevelSelection::Index(0)
    };

    let mut app = App::new();

    app.insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_plugin(LdtkPlugin)
        .add_plugin(SpriteSheetAnimationPlugin)
        .add_plugin(FromComponentAnimator::<willo::PlayerAnimationState>::new())
        .add_event::<animation::AnimationEvent>()
        .add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::LevelTransition)
                .with_collection::<AssetHolder>(),
        )
        .add_plugin(ui::UiPlugin)
        .add_plugin(level_select::LevelSelectPlugin)
        .add_event::<willo::PlayerMovementEvent>()
        .add_event::<history::HistoryCommands>()
        .add_event::<gameplay::DeathEvent>()
        .add_event::<gameplay::GoalEvent>()
        .add_plugin(event_scheduler::EventSchedulerPlugin::<
            gameplay::LevelCardEvent,
        >::new())
        .insert_resource(LdtkSettings {
            set_clear_color: SetClearColor::FromEditorBackground,
            ..default()
        })
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(level_selection)
        .insert_resource(resources::GoalGhostSettings::NORMAL)
        .insert_resource(resources::RewindSettings::NORMAL)
        .insert_resource(resources::PlayZonePortion(0.75))
        .add_startup_system(gameplay::transitions::spawn_camera)
        .add_startup_system(gameplay::transitions::spawn_ui_root)
        .add_startup_system(gameplay::transitions::schedule_first_level_card)
        .add_system_to_stage(CoreStage::PreUpdate, sugar::make_ui_visible)
        .add_enter_system(
            GameState::Gameplay,
            gameplay::transitions::fit_camera_around_play_zone_padded,
        )
        .add_system(
            gameplay::transitions::fit_camera_around_play_zone_padded
                .run_not_in_state(GameState::AssetLoading)
                .run_on_event::<bevy::window::WindowResized>(),
        )
        .add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::LevelTransition)
                .with_system(gameplay::transitions::spawn_level_card)
                .with_system(gameplay::transitions::spawn_gravestone_body)
                .with_system(gameplay::transitions::spawn_control_display)
                .with_system(gameplay::transitions::load_next_level)
                .with_system(gameplay::transitions::level_card_update)
                .with_system(gameplay::transitions::spawn_goal_ghosts)
                .into(),
        )
        .add_system(
            sokoban::move_table_update
                .run_in_state(GameState::Gameplay)
                .before(SystemLabels::Input),
        )
        .add_system(
            willo::player_state_input
                .run_in_state(GameState::Gameplay)
                .label(SystemLabels::Input)
                .before(history::FlushHistoryCommands),
        )
        .add_system(
            sokoban::perform_grid_coords_movement
                .run_in_state(GameState::Gameplay)
                .label(SystemLabels::MoveTableUpdate)
                .before(from_component::FromComponentLabel),
        )
        .add_system(
            gameplay::systems::check_death
                .run_in_state(GameState::Gameplay)
                .label(SystemLabels::CheckDeath)
                .after(history::FlushHistoryCommands),
        )
        .add_system(
            history::flush_history_commands::<GridCoords>
                .run_in_state(GameState::Gameplay)
                .label(history::FlushHistoryCommands),
        )
        .add_system(
            gameplay::systems::check_goal
                .run_in_state(GameState::Gameplay)
                .after(SystemLabels::CheckDeath),
        )
        .add_system(
            willo::move_player_by_table
                .run_in_state(GameState::Gameplay)
                .after(SystemLabels::MoveTableUpdate)
                .after(history::FlushHistoryCommands),
        )
        .add_system(gameplay::transitions::spawn_death_card.run_in_state(GameState::Gameplay))
        .add_system(gameplay::systems::update_control_display.run_in_state(GameState::Gameplay))
        // Systems with potential easing end/beginning collisions cannot be in CoreStage::Update
        // see https://github.com/vleue/bevy_easings/issues/23
        .add_system_to_stage(
            CoreStage::PostUpdate,
            willo::reset_player_easing
                .run_not_in_state(GameState::AssetLoading)
                .before("ease_movement"),
        )
        .add_system_to_stage(
            CoreStage::PostUpdate,
            sokoban::ease_movement
                .run_not_in_state(GameState::AssetLoading)
                .label("ease_movement"),
        )
        .add_system(sugar::goal_ghost_animation.run_not_in_state(GameState::AssetLoading))
        .add_system(sugar::goal_ghost_event_sugar.run_not_in_state(GameState::AssetLoading))
        .add_system(sugar::animate_grass_system.run_not_in_state(GameState::AssetLoading))
        .add_system(willo::play_death_animations.run_not_in_state(GameState::AssetLoading))
        .add_system(willo::history_sugar.run_not_in_state(GameState::AssetLoading))
        .register_ldtk_entity::<willo::PlayerBundle>("Willo")
        .register_ldtk_entity::<bundles::InputBlockBundle>("W")
        .register_ldtk_entity::<bundles::InputBlockBundle>("A")
        .register_ldtk_entity::<bundles::InputBlockBundle>("S")
        .register_ldtk_entity::<bundles::InputBlockBundle>("D")
        .register_ldtk_entity::<bundles::GoalBundle>("Goal")
        .register_ldtk_entity::<bundles::MoveTableBundle>("Table")
        .register_ldtk_entity::<bundles::GrassBundle>("Grass")
        .register_ldtk_int_cell::<sokoban::WallBundle>(1)
        .register_ldtk_int_cell::<sokoban::WallBundle>(3)
        .register_ldtk_int_cell::<sokoban::WallBundle>(4)
        .register_ldtk_int_cell::<bundles::ExorcismBlockBundle>(2)
        .register_ldtk_int_cell::<bundles::ExorcismBlockBundle>(2);

    #[cfg(feature = "hot")]
    {
        app.add_startup_system(enable_hot_reloading);
    }

    #[cfg(feature = "inspector")]
    {
        app.add_plugin(WorldInspectorPlugin::new());
    }

    app.run()
}

#[derive(Debug, Default, AssetCollection)]
pub struct AssetHolder {
    #[asset(path = "levels/willos-graveyard.ldtk")]
    pub ldtk: Handle<LdtkAsset>,
    #[asset(path = "fonts/WayfarersToyBoxRegular-gxxER.ttf")]
    pub font: Handle<Font>,
    #[asset(path = "textures/button-underline.png")]
    pub button_underline: Handle<Image>,
    #[asset(path = "textures/button-radial.png")]
    pub button_radial: Handle<Image>,
    #[asset(path = "sfx/victory.wav")]
    pub victory_sound: Handle<AudioSource>,
    #[asset(path = "sfx/push.wav")]
    pub push_sound: Handle<AudioSource>,
    #[asset(path = "sfx/undo.wav")]
    pub undo_sound: Handle<AudioSource>,
    #[asset(path = "textures/tarot.png")]
    pub tarot_sheet: Handle<Image>,
}

#[cfg(feature = "hot")]
pub fn enable_hot_reloading(asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
}
