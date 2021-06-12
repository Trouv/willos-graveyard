mod gameplay;

use bevy::prelude::*;

pub const UNIT_LENGTH: f32 = 32.;

pub struct LevelSize {
    size: IVec2,
}

impl LevelSize {
    fn new(&self, size: IVec2) -> Self {
        LevelSize { size: size }
    }
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(gameplay::create_camera.system())
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
) {
    commands.insert_resource(SpriteHandles {
        up: materials.add(assets.load("assets/textures/up.png").into()),
        left: materials.add(assets.load("assets/textures/left.png").into()),
        right: materials.add(assets.load("assets/textures/right.png").into()),
        down: materials.add(assets.load("assets/textures/down.png").into()),
        goal: materials.add(assets.load("assets/textures/goal.png").into()),
        player: materials.add(assets.load("assets/textures/player.png").into()),
        wall: materials.add(assets.load("assets/textures/wall.png").into()),
        w_block: materials.add(assets.load("assets/textures/w_block.png").into()),
        a_block: materials.add(assets.load("assets/textures/a_block.png").into()),
        s_block: materials.add(assets.load("assets/textures/s_block.png").into()),
        d_block: materials.add(assets.load("assets/textures/d_block.png").into()),
    });
}
