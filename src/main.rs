mod animation;
mod event_scheduler;
mod from_component;
mod gameplay;
mod resources;

use animation::{FromComponentAnimator, SpriteSheetAnimationPlugin};
use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_ecs_ldtk::prelude::*;
use rand::Rng;

pub const UNIT_LENGTH: f32 = 32.;

pub const PLAY_ZONE_RATIO: Size<i32> = Size {
    width: 4,
    height: 3,
};

pub const ASPECT_RATIO: Size<i32> = Size {
    width: 16,
    height: 9,
};

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

    app.add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_plugin(LdtkPlugin)
        .add_plugin(SpriteSheetAnimationPlugin)
        .add_plugin(FromComponentAnimator::<
            gameplay::components::PlayerAnimationState,
        >::new())
        .add_plugin(FromComponentAnimator::<gameplay::components::DeathHoleState>::new())
        .add_plugin(FromComponentAnimator::<gameplay::components::DemonArmsState>::new())
        .add_event::<animation::AnimationEvent>()
        .add_event::<gameplay::PlayerMovementEvent>()
        .add_event::<gameplay::HistoryEvent>()
        .add_event::<gameplay::DeathEvent>()
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
        .insert_resource(resources::GoalGhostSettings::NORMAL)
        .add_startup_system_to_stage(StartupStage::PreStartup, sprite_load)
        .add_startup_system_to_stage(StartupStage::PreStartup, sound_load)
        .add_startup_system(gameplay::transitions::world_setup)
        .add_startup_system(gameplay::transitions::spawn_ui_root)
        .add_startup_system(gameplay::transitions::schedule_first_level_card)
        .add_startup_system(animation::load_death_animations)
        .add_system(gameplay::systems::player_state_input.label(SystemLabels::Input))
        .add_system(gameplay::systems::move_table_update.before(SystemLabels::Input))
        .add_system(
            gameplay::systems::perform_grid_coords_movement.label(SystemLabels::MoveTableUpdate),
        )
        .add_system(gameplay::systems::store_current_position.before(SystemLabels::MoveTableUpdate))
        .add_system(gameplay::systems::check_death.label(SystemLabels::CheckDeath))
        .add_system(gameplay::systems::check_goal.after(SystemLabels::CheckDeath))
        .add_system(gameplay::systems::move_player_by_table.after(SystemLabels::MoveTableUpdate))
        .add_system(gameplay::systems::rewind)
        .add_system(gameplay::systems::reset)
        .add_system(gameplay::systems::ease_movement)
        .add_system(gameplay::systems::update_control_display)
        .add_system(animation::goal_ghost_animation)
        .add_system(gameplay::transitions::spawn_gravestone_body)
        .add_system(gameplay::transitions::spawn_control_display)
        .add_system(gameplay::transitions::spawn_death_card)
        .add_system(gameplay::transitions::spawn_level_card)
        .add_system(gameplay::transitions::load_next_level)
        .add_system(gameplay::transitions::level_card_update)
        .add_system(gameplay::transitions::fit_camera_around_play_zone_padded)
        .add_system(gameplay::transitions::spawn_goal_ghosts)
        .add_system(gameplay::systems::animate_grass_system)
        .add_system(animation::play_death_animations)
        .add_system(animation::despawn_death_animations)
        .register_ldtk_entity::<gameplay::bundles::PlayerBundle>("Willo")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("W")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("A")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("S")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("D")
        .register_ldtk_entity::<gameplay::bundles::GoalBundle>("Goal")
        .register_ldtk_entity::<gameplay::bundles::MoveTableBundle>("Table")
        .register_ldtk_entity::<gameplay::bundles::GrassBundle>("Grass")
        .register_ldtk_int_cell::<gameplay::bundles::WallBundle>(1)
        .register_ldtk_int_cell::<gameplay::bundles::WallBundle>(3)
        .register_ldtk_int_cell::<gameplay::bundles::WallBundle>(4)
        .register_ldtk_int_cell::<gameplay::bundles::ExorcismBlockBundle>(2)
        .register_ldtk_int_cell::<gameplay::bundles::ExorcismBlockBundle>(2);

    #[cfg(feature = "hot")]
    {
        app.add_startup_system(enable_hot_reloading);
    }

    app.run()
}

pub struct SpriteHandles {
    pub grass_plain: Handle<TextureAtlas>,
    pub grass_tufts: Handle<TextureAtlas>,
    pub grass_stone: Handle<TextureAtlas>,
}

impl SpriteHandles {
    pub fn get_rand_grass(&self) -> Handle<TextureAtlas> {
        let mut rng = rand::thread_rng();
        let chance = rng.gen::<f32>();
        if chance < 0.9 {
            self.grass_plain.clone_weak()
        } else if chance > 0.9 && chance < 0.95 {
            self.grass_stone.clone_weak()
        } else {
            self.grass_tufts.clone_weak()
        }
    }
}

pub fn sprite_load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let grass_plain_handle = assets.load("textures/grass_plain.png");
    let grass_plain_atlas =
        TextureAtlas::from_grid(grass_plain_handle, Vec2::new(32.0, 32.0), 4, 1);
    let grass_stone_handle = assets.load("textures/grass_stone.png");
    let grass_stone_atlas =
        TextureAtlas::from_grid(grass_stone_handle, Vec2::new(32.0, 32.0), 4, 1);
    let grass_tufts_handle = assets.load("textures/grass_tufts.png");
    let grass_tufts_atlas =
        TextureAtlas::from_grid(grass_tufts_handle, Vec2::new(32.0, 32.0), 4, 1);
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(SpriteHandles {
        grass_plain: texture_atlases.add(grass_plain_atlas),
        grass_tufts: texture_atlases.add(grass_tufts_atlas),
        grass_stone: texture_atlases.add(grass_stone_atlas),
    });
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
