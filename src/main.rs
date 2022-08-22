// these two lints are triggered by normal system code a lot
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod animation;
mod bundles;
mod event_scheduler;
mod from_component;
mod gameplay;
mod goal_ghost;
mod history;
mod resources;
mod sugar;
mod utils;

use animation::{FromComponentAnimator, SpriteSheetAnimationPlugin};
use bevy::{prelude::*, render::texture::ImageSettings};

use bevy_easings::EasingsPlugin;
use bevy_ecs_ldtk::prelude::*;
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
pub enum LevelState {
    Gameplay,
    Inbetween,
}

fn main() {
    let mut level_num = 0;
    if std::env::args().count() > 1 {
        level_num = std::env::args().last().unwrap().parse::<usize>().unwrap();
    }

    let mut app = App::new();

    app.insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_plugin(LdtkPlugin)
        .add_plugin(SpriteSheetAnimationPlugin)
        .add_plugin(FromComponentAnimator::<sugar::PlayerAnimationState>::new())
        .add_event::<animation::AnimationEvent>()
        .add_event::<gameplay::PlayerMovementEvent>()
        .add_event::<history::HistoryCommands>()
        .add_event::<gameplay::DeathEvent>()
        .add_event::<gameplay::GoalEvent>()
        .add_plugin(event_scheduler::EventSchedulerPlugin::<
            gameplay::LevelCardEvent,
        >::new())
        .insert_resource(LdtkSettings {
            set_clear_color: SetClearColor::FromLevelBackground,
            ..default()
        })
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(LevelSelection::Index(level_num))
        .insert_resource(LevelState::Inbetween)
        .insert_resource(goal_ghost::GoalGhostSettings::NORMAL)
        .insert_resource(resources::RewindSettings::NORMAL)
        .insert_resource(resources::PlayZonePortion(0.75))
        .add_startup_system_to_stage(StartupStage::PreStartup, sound_load)
        .add_startup_system(gameplay::transitions::world_setup)
        .add_startup_system(gameplay::transitions::spawn_ui_root)
        .add_startup_system(gameplay::transitions::schedule_first_level_card)
        .add_system_to_stage(CoreStage::PreUpdate, sugar::make_ui_visible)
        .add_system(
            gameplay::systems::player_state_input
                .label(SystemLabels::Input)
                .before(history::FlushHistoryCommands),
        )
        .add_system(gameplay::systems::move_table_update.before(SystemLabels::Input))
        .add_system(
            gameplay::systems::perform_grid_coords_movement
                .label(SystemLabels::MoveTableUpdate)
                .before(from_component::FromComponentLabel),
        )
        .add_system(
            gameplay::systems::check_death
                .label(SystemLabels::CheckDeath)
                .after(history::FlushHistoryCommands),
        )
        .add_system(gameplay::systems::check_goal.after(SystemLabels::CheckDeath))
        .add_system(
            history::flush_history_commands::<GridCoords>.label(history::FlushHistoryCommands),
        )
        .add_system(
            gameplay::systems::move_player_by_table
                .after(SystemLabels::MoveTableUpdate)
                .after(history::FlushHistoryCommands),
        )
        .add_system(sugar::ease_movement)
        .add_system(animation::kill_one_shot_sprites)
        .add_system_to_stage(CoreStage::PreUpdate, sugar::reset_player_easing)
        .add_system(gameplay::systems::update_control_display)
        .add_system(gameplay::transitions::spawn_gravestone_body)
        .add_system(gameplay::transitions::spawn_control_display)
        .add_system(gameplay::transitions::spawn_death_card)
        .add_system(gameplay::transitions::spawn_level_card)
        .add_system(gameplay::transitions::load_next_level)
        .add_system(gameplay::transitions::level_card_update)
        .add_system(gameplay::transitions::fit_camera_around_play_zone_padded)
        .add_system(goal_ghost::spawn_goal_ghosts)
        .add_system(goal_ghost::goal_ghost_animation)
        .add_system(goal_ghost::punctuation)
        .add_system(goal_ghost::goal_ghost_event_sugar)
        .add_system(sugar::animate_grass_system)
        .add_system(sugar::play_death_animations)
        .add_system(sugar::history_sugar)
        .register_ldtk_entity::<bundles::PlayerBundle>("Willo")
        .register_ldtk_entity::<bundles::InputBlockBundle>("W")
        .register_ldtk_entity::<bundles::InputBlockBundle>("A")
        .register_ldtk_entity::<bundles::InputBlockBundle>("S")
        .register_ldtk_entity::<bundles::InputBlockBundle>("D")
        .register_ldtk_entity::<bundles::GoalBundle>("Goal")
        .register_ldtk_entity::<bundles::MoveTableBundle>("Table")
        .register_ldtk_entity::<bundles::GrassBundle>("Grass")
        .register_ldtk_int_cell::<bundles::WallBundle>(1)
        .register_ldtk_int_cell::<bundles::WallBundle>(3)
        .register_ldtk_int_cell::<bundles::WallBundle>(4)
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

pub struct SoundEffects {
    pub victory: Handle<AudioSource>,
    pub push: Handle<AudioSource>,
    pub undo: Handle<AudioSource>,
}

pub fn sound_load(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(SoundEffects {
        victory: assets.load("sfx/victory.wav"),
        push: assets.load("sfx/push.wav"),
        undo: assets.load("sfx/undo.wav"),
    })
}

#[cfg(feature = "hot")]
pub fn enable_hot_reloading(asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
}
