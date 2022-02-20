mod gameplay;
mod utils;

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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct LevelSize {
    size: IVec2,
}

impl LevelSize {
    fn new(size: IVec2) -> Self {
        LevelSize { size }
    }
}

pub const LEVEL_ORDER: [&str; 6] = [
    "who-put-this.txt",
    "getting-used-to.txt",
    "set-the-table.txt",
    "claustrophobic.txt",
    "flip-the-table.txt",
    "stuck.txt",
];

pub struct LevelNum(usize);

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
        .insert_resource(LevelEntities(Vec::new()))
        .insert_resource(LevelState::Inbetween)
        .add_startup_system_to_stage(StartupStage::PreStartup, sprite_load)
        .add_startup_system_to_stage(StartupStage::PreStartup, sound_load)
        .add_startup_system(gameplay::transitions::world_setup)
        // .add_startup_system(gameplay::transitions::simple_camera_setup)
        //.add_startup_system(gameplay::transitions::test_level_setup)
        //.add_system(
        //gameplay::transitions::load_level
        //.chain(gameplay::transitions::create_camera)
        //.label(SystemLabels::LoadAssets),
        //)
        //.add_system(gameplay::transitions::spawn_table_edges)
        //.add_system(
        //gameplay::systems::simple_movement
        //
        //.label(SystemLabels::Input),
        //)
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
        .add_system(gameplay::systems::animate_grass_system)
        //.add_system(gameplay::systems::render_rope.before(SystemLabels::MoveTableUpdate))
        .register_ldtk_entity::<gameplay::bundles::PlayerBundle>("Willo")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("W")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("A")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("S")
        .register_ldtk_entity::<gameplay::bundles::InputBlockBundle>("D")
        .register_ldtk_entity::<gameplay::bundles::GoalBundle>("Goal")
        .register_ldtk_entity::<gameplay::bundles::MoveTableBundle>("Table")
        .register_ldtk_int_cell::<gameplay::bundles::WallBundle>(1)
        .run()
}

pub struct SpriteHandles {
    pub up: Handle<Image>,
    pub left: Handle<Image>,
    pub right: Handle<Image>,
    pub down: Handle<Image>,
    pub goal: Handle<Image>,
    pub player: Handle<Image>,
    pub wall: Handle<Image>,
    pub bush: Handle<Image>,
    pub rope: Handle<Image>,
    pub w_block: Vec<Handle<Image>>,
    pub a_block: Vec<Handle<Image>>,
    pub s_block: Vec<Handle<Image>>,
    pub d_block: Vec<Handle<Image>>,
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
        TextureAtlas::from_grid(grass_plain_handle, Vec2::new(32.0, 32.0), 4, 1);
    let grass_stone_handle = assets.load("textures/grass_stone.png");
    let grass_stone_atlas =
        TextureAtlas::from_grid(grass_stone_handle, Vec2::new(32.0, 32.0), 4, 1);
    let grass_tufts_handle = assets.load("textures/grass_tufts.png");
    let grass_tufts_atlas =
        TextureAtlas::from_grid(grass_tufts_handle, Vec2::new(32.0, 32.0), 4, 1);
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(SpriteHandles {
        up: assets.load("textures/up.png"),
        left: assets.load("textures/left.png"),
        right: assets.load("textures/right.png"),
        down: assets.load("textures/down.png"),
        goal: assets.load("textures/goal.png"),
        player: assets.load("textures/player.png"),
        wall: assets.load("textures/fence.png"),
        bush: assets.load("textures/bush.png"),
        rope: assets.load("textures/rope.png"),
        w_block: vec![w_0, w_1],
        a_block: vec![a_0, a_1],
        s_block: vec![s_0, s_1],
        d_block: vec![d_0, d_1],
        grass_plain: texture_atlases.add(grass_plain_atlas),
        grass_tufts: texture_atlases.add(grass_tufts_atlas),
        grass_stone: texture_atlases.add(grass_stone_atlas),
    });

    writer.send(gameplay::LevelCompleteEvent);
}

pub struct LevelEntities(Vec<Entity>);

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
