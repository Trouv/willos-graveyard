mod gameplay;
mod utils;

use bevy::{prelude::*, sprite::TextureAtlasBuilder};
use bevy_easings::EasingsPlugin;

pub const UNIT_LENGTH: f32 = 32.;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, SystemLabel)]
pub enum SystemLabels {
    LoadAssets,
    Input,
    MoveTableUpdate,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct LevelSize {
    size: IVec2,
}

impl LevelSize {
    fn new(size: IVec2) -> Self {
        LevelSize { size: size }
    }
}

pub const LEVEL_ORDER: [&str; 2] = ["hello.txt", "stuck.txt"];

pub struct LevelNum(usize);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum LevelState {
    Gameplay,
    Inbetween,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_event::<gameplay::MovementEvent>()
        .add_event::<gameplay::ActionEvent>()
        .add_event::<gameplay::LevelCompleteEvent>()
        .add_event::<gameplay::CardUpEvent>()
        .add_event::<gameplay::LevelStartEvent>()
        .insert_resource(LevelSize::new(IVec2::new(16, 9)))
        .insert_resource(LevelNum(0))
        .insert_resource(LevelEntities(Vec::new()))
        .insert_resource(LevelState::Inbetween)
        .add_startup_system_to_stage(StartupStage::PreStartup, sprite_load.system())
        // .add_startup_system(gameplay::transitions::simple_camera_setup.system())
        //.add_startup_system(gameplay::transitions::test_level_setup.system())
        .add_system(
            gameplay::transitions::load_level
                .system()
                .chain(gameplay::transitions::create_camera.system())
                .label(SystemLabels::LoadAssets),
        )
        .add_system(gameplay::transitions::spawn_table_edges.system())
        //.add_system(
        //gameplay::systems::simple_movement
        //.system()
        //.label(SystemLabels::Input),
        //)
        .add_system(
            gameplay::systems::player_state_input
                .system()
                .label(SystemLabels::Input),
        )
        .add_system(
            gameplay::systems::move_table_update
                .system()
                .before(SystemLabels::Input),
        )
        .add_system(
            gameplay::systems::perform_tile_movement
                .system()
                .label(SystemLabels::MoveTableUpdate),
        )
        .add_system(
            gameplay::systems::store_current_position
                .system()
                .before(SystemLabels::MoveTableUpdate),
        )
        .add_system(
            gameplay::systems::check_goal
                .system()
                .after(SystemLabels::MoveTableUpdate),
        )
        .add_system(
            gameplay::systems::move_player_by_table
                .system()
                .after(SystemLabels::MoveTableUpdate),
        )
        .add_system(gameplay::systems::rewind.system())
        .add_system(gameplay::systems::reset.system())
        .add_system(gameplay::systems::ease_movement.system())
        .add_system(gameplay::transitions::spawn_level_card.system())
        .add_system(gameplay::transitions::level_card_update.system())
        .run()
}

pub struct SpriteHandles {
    pub up: Handle<ColorMaterial>,
    pub left: Handle<ColorMaterial>,
    pub right: Handle<ColorMaterial>,
    pub down: Handle<ColorMaterial>,
    pub goal: Handle<ColorMaterial>,
    pub player: Handle<ColorMaterial>,
    pub wall: Handle<ColorMaterial>,
    pub w_block: Vec<Handle<ColorMaterial>>,
    pub a_block: Vec<Handle<ColorMaterial>>,
    pub s_block: Vec<Handle<ColorMaterial>>,
    pub d_block: Vec<Handle<ColorMaterial>>,
    pub grass_plain: Handle<TextureAtlas>,
    pub grass_tufts: Handle<TextureAtlas>,
    pub grass_stone: Handle<TextureAtlas>,
}

impl SpriteHandles {
    pub fn get_rand_grass(&self) {
        false;
    }
}

pub fn sprite_load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut writer: EventWriter<gameplay::LevelCompleteEvent>,
) {
    let w_0 = assets.load("textures/w_0.png");
    let w_1 = assets.load("textures/w_1.png");
    let a_0 = assets.load("textures/a_0.png");
    let a_1 = assets.load("textures/a_1.png");
    let s_0 = assets.load("textures/s_0.png");
    let s_1 = assets.load("textures/s_1.png");
    let d_0 = assets.load("textures/d_0.png");
    let d_1 = assets.load("textures/d_1.png");
    let grass_plain_handle = assets.load("textures/grass_plain.png");
    let grass_plain_atlas =
        TextureAtlas::from_grid(grass_plain_handle, Vec2::new(32.0, 32.0), 1, 1);
    let grass_stone_handle = assets.load("textures/grass_stone.png");
    let grass_stone_atlas =
        TextureAtlas::from_grid(grass_stone_handle, Vec2::new(32.0, 32.0), 4, 1);
    let grass_tufts_handle = assets.load("textures/grass_tufts.png");
    let grass_tufts_atlas =
        TextureAtlas::from_grid(grass_tufts_handle, Vec2::new(32.0, 32.0), 4, 1);
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(SpriteHandles {
        up: materials.add(assets.load("textures/up.png").into()),
        left: materials.add(assets.load("textures/left.png").into()),
        right: materials.add(assets.load("textures/right.png").into()),
        down: materials.add(assets.load("textures/down.png").into()),
        goal: materials.add(assets.load("textures/goal.png").into()),
        player: materials.add(assets.load("textures/player.png").into()),
        wall: materials.add(assets.load("textures/wall.png").into()),
        w_block: vec![materials.add(w_0.into()), materials.add(w_1.into())],
        a_block: vec![materials.add(a_0.into()), materials.add(a_1.into())],
        s_block: vec![materials.add(s_0.into()), materials.add(s_1.into())],
        d_block: vec![materials.add(d_0.into()), materials.add(d_1.into())],
        grass_plain: texture_atlases.add(grass_plain_atlas),
        grass_tufts: texture_atlases.add(grass_tufts_atlas),
        grass_stone: texture_atlases.add(grass_stone_atlas),
    });

    writer.send(gameplay::LevelCompleteEvent);
}

pub struct LevelEntities(Vec<Entity>);
