mod gameplay;

use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_ecs_ldtk::prelude::*;
use rand::Rng;

pub const UNIT_LENGTH: f32 = 32.;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, SystemLabel)]
pub enum SystemLabels {
    LoadAssets,
    Input,
    MoveTableUpdate,
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

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_plugin(LdtkPlugin)
        .add_event::<gameplay::PlayerMovementEvent>()
        .add_event::<gameplay::ActionEvent>()
        .add_event::<gameplay::LevelCompleteEvent>()
        .add_event::<gameplay::CardUpEvent>()
        .add_event::<gameplay::LevelStartEvent>()
        .insert_resource(LevelSelection::Index(level_num))
        .insert_resource(LevelState::Inbetween)
        .add_startup_system_to_stage(StartupStage::PreStartup, sprite_load)
        .add_startup_system_to_stage(StartupStage::PreStartup, sound_load)
        .add_startup_system(gameplay::transitions::world_setup)
        .add_system(gameplay::systems::player_state_input.label(SystemLabels::Input))
        .add_system(gameplay::systems::move_table_update.before(SystemLabels::Input))
        .add_system(
            gameplay::systems::perform_grid_coords_movement.label(SystemLabels::MoveTableUpdate),
        )
        .add_system(gameplay::systems::store_current_position.before(SystemLabels::MoveTableUpdate))
        .add_system(gameplay::systems::check_goal.after(SystemLabels::MoveTableUpdate))
        .add_system(gameplay::systems::move_player_by_table.after(SystemLabels::MoveTableUpdate))
        .add_system(gameplay::systems::rewind)
        .add_system(gameplay::systems::reset)
        .add_system(gameplay::systems::ease_movement)
        .add_system(gameplay::transitions::spawn_level_card)
        .add_system(gameplay::transitions::level_card_update)
        .add_system(gameplay::transitions::fit_camera_around_play_zone_padded)
        .add_system(gameplay::systems::animate_grass_system)
        .register_ldtk_entity::<gameplay::bundles::PlayerBundle>("Willo")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("W")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("A")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("S")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("D")
        .register_ldtk_entity::<gameplay::bundles::GoalBundle>("Goal")
        .register_ldtk_entity::<gameplay::bundles::MoveTableBundle>("Table")
        .register_ldtk_entity::<gameplay::bundles::GrassBundle>("Grass")
        .register_ldtk_int_cell::<gameplay::bundles::WallBundle>(1)
        .run()
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
    mut writer: EventWriter<gameplay::LevelCompleteEvent>,
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

    writer.send(gameplay::LevelCompleteEvent);
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
