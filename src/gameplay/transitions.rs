use crate::{
    gameplay::{bundles::*, components::*, Direction, DIRECTION_ORDER},
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
    let player = commands
        .spawn_bundle(PlayerBundle::new(IVec2::new(1, 0), &sprite_handles))
        .id();
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
    commands.spawn_bundle(InputBlockBundle::new(
        Direction::Right,
        IVec2::new(2, 2),
        &sprite_handles,
    ));
    commands.spawn_bundle(MoveTableBundle::new(
        player,
        IVec2::new(1, 7),
        &sprite_handles,
    ));
    commands.insert_resource(sprite_handles);
}

pub fn spawn_table_edges(
    mut commands: Commands,
    table_query: Query<&Tile, Added<MoveTable>>,
    sprite_handles: Res<SpriteHandles>,
) {
    for tile in table_query.iter() {
        for (i, direction) in DIRECTION_ORDER.iter().enumerate() {
            commands.spawn_bundle(DirectionTileBundle::new(
                *direction,
                tile.coords + (i as i32 + 1) * IVec2::from(Direction::Right),
                &sprite_handles,
            ));
            commands.spawn_bundle(DirectionTileBundle::new(
                *direction,
                tile.coords + (i as i32 + 1) * IVec2::from(Direction::Down),
                &sprite_handles,
            ));
        }
    }
}

pub fn create_camera(mut commands: Commands, level_size: Res<LevelSize>) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    let scale =
        if (9.0 / 16.0) > ((level_size.size.y as f32 + 2.) / (level_size.size.x as f32 + 2.)) {
            (level_size.size.x as f32 + 2.) / UNIT_LENGTH / 1.25
        } else {
            (level_size.size.y as f32 + 2.) / UNIT_LENGTH / 1.25 * (16. / 9.)
        };
    camera_bundle.transform.translation = Vec3::new(
        ((level_size.size.x as f32) * UNIT_LENGTH) / 2. - (UNIT_LENGTH / 2.),
        ((level_size.size.y as f32) * UNIT_LENGTH) / 2. - (UNIT_LENGTH / 2.),
        camera_bundle.transform.translation.z,
    );
    camera_bundle.orthographic_projection.scale = scale;
    commands.spawn().insert_bundle(camera_bundle);
}
