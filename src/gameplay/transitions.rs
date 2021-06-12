use crate::{
    gameplay::{bundles::*, Direction},
    LevelSize, SpriteHandles, UNIT_LENGTH,
};
use bevy::prelude::*;

pub fn simple_camera_setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

pub fn test_level_setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let sprite_handles = SpriteHandles {
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
    };
    commands.spawn_bundle(WallBundle::new(IVec2::new(0, 0), &sprite_handles));
    commands.spawn_bundle(PlayerBundle::new(IVec2::new(1, 0), &sprite_handles));
    commands.spawn_bundle(DirectionTileBundle::new(
        Direction::Up,
        IVec2::new(0, 1),
        &sprite_handles,
    ));
    commands.spawn_bundle(InputBlockBundle::new(
        Direction::Up,
        IVec2::new(1, 1),
        &sprite_handles,
    ));
    commands.insert_resource(sprite_handles);
}

pub fn create_camera(mut commands: Commands, level_size: Res<LevelSize>) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    let scale = level_size.size.max_element() as f32;
    camera_bundle.transform.translation =
        Vec3::new((16.0 * UNIT_LENGTH) / 2.0, (9.0 * UNIT_LENGTH) / 2.0, 0.0);
    camera_bundle.orthographic_projection.far = 1000.0 / scale;
    camera_bundle.orthographic_projection.scale = scale;
    commands.spawn().insert_bundle(camera_bundle);
}
