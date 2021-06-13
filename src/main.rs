mod gameplay;
mod utils;

use bevy::prelude::*;
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

    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(EasingsPlugin)
        .add_event::<gameplay::MovementEvent>()
        .add_event::<gameplay::ActionEvent>()
        .add_event::<gameplay::LevelCompleteEvent>()
        .add_event::<gameplay::CardUpEvent>()
        .add_event::<gameplay::LevelStartEvent>()
        .insert_resource(LevelSize::new(IVec2::new(16, 9)))
        .insert_resource(LevelNum(level_num))
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
    pub w_block: Handle<ColorMaterial>,
    pub a_block: Handle<ColorMaterial>,
    pub s_block: Handle<ColorMaterial>,
    pub d_block: Handle<ColorMaterial>,
}

pub fn sprite_load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut writer: EventWriter<gameplay::LevelCompleteEvent>,
) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands.insert_resource(SpriteHandles {
        up: materials.add(assets.load("textures/up.png").into()),
        left: materials.add(assets.load("textures/left.png").into()),
        right: materials.add(assets.load("textures/right.png").into()),
        down: materials.add(assets.load("textures/down.png").into()),
        goal: materials.add(assets.load("textures/goal.png").into()),
        player: materials.add(assets.load("textures/player.png").into()),
        wall: materials.add(assets.load("textures/wall.png").into()),
        w_block: materials.add(assets.load("textures/w_block.png").into()),
        a_block: materials.add(assets.load("textures/a_block.png").into()),
        s_block: materials.add(assets.load("textures/s_block.png").into()),
        d_block: materials.add(assets.load("textures/d_block.png").into()),
    });

    writer.send(gameplay::LevelCompleteEvent);
}

pub struct LevelEntities(Vec<Entity>);
