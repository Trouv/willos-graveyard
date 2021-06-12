use crate::{
    gameplay::{bundles::*, components::*, Direction, DIRECTION_ORDER},
    LevelNum, LevelSize, SpriteHandles, UNIT_LENGTH,
};
use bevy::prelude::*;

pub fn simple_camera_setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

pub fn load_level(
    mut commands: Commands,
    sprite_handles: Res<SpriteHandles>,
    level_num: Res<LevelNum>,
) {
}

pub fn test_level_setup(mut commands: Commands, sprite_handles: Res<SpriteHandles>) {
    commands.spawn_bundle(WallBundle::new(IVec2::new(0, 0), &sprite_handles));
    let player = commands
        .spawn_bundle(PlayerBundle::new(IVec2::new(1, 0), &sprite_handles))
        .id();
    commands.spawn_bundle(InputBlockBundle::new(
        Direction::Up,
        IVec2::new(2, 6),
        &sprite_handles,
    ));
    commands.spawn_bundle(InputBlockBundle::new(
        Direction::Left,
        IVec2::new(3, 5),
        &sprite_handles,
    ));
    commands.spawn_bundle(InputBlockBundle::new(
        Direction::Down,
        IVec2::new(4, 4),
        &sprite_handles,
    ));
    commands.spawn_bundle(InputBlockBundle::new(
        Direction::Right,
        IVec2::new(5, 3),
        &sprite_handles,
    ));
    commands.spawn_bundle(MoveTableBundle::new(
        player,
        IVec2::new(1, 7),
        &sprite_handles,
    ));
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
