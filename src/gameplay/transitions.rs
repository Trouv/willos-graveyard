use crate::{
    gameplay::{bundles::*, components::*, Direction, DIRECTION_ORDER},
    utils::application_root_dir,
    LevelNum, LevelSize, SpriteHandles, LEVEL_ORDER, UNIT_LENGTH,
};
use bevy::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub fn simple_camera_setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn file_to_tile_coords(i: usize, j: usize, height: usize) -> IVec2 {
    IVec2::new(j as i32, height as i32 - i as i32 - 1)
}

pub fn load_level(
    mut commands: Commands,
    sprite_handles: Res<SpriteHandles>,
    level_num: Res<LevelNum>,
) {
    let level_path = application_root_dir()
        .unwrap()
        .join(Path::new("assets/levels/"))
        .join(Path::new(LEVEL_ORDER[level_num.0]));

    let mut lines =
        BufReader::new(File::open(level_path).expect("level file should exist")).lines();

    let title = lines.next().unwrap();

    let line_strings = lines.map(|x| x.unwrap()).collect::<Vec<String>>();

    let mut willow = None;
    let mut chester = None;
    let mut width = 0;
    let mut height = 0;
    // Player pass, and get width and height
    for (i, line) in line_strings.clone().iter().enumerate() {
        for (j, tile_char) in line.chars().enumerate() {
            if tile_char == 'I' {
                willow = Some((i, j))
            } else if tile_char == 'C' {
                chester = Some((i, j))
            }
            if j + 1 > width {
                width = j + 1
            };
        }
        if i + 1 > height {
            height = i + 1
        };
    }

    let willow_id = match willow {
        Some(w) => Some(
            commands
                .spawn_bundle(PlayerBundle::new(
                    file_to_tile_coords(w.0, w.1, height),
                    &&sprite_handles,
                ))
                .id(),
        ),
        None => None,
    };
    let chester_id = match chester {
        Some(c) => Some(
            commands
                .spawn_bundle(PlayerBundle::new(
                    file_to_tile_coords(c.0, c.1, height),
                    &&sprite_handles,
                ))
                .id(),
        ),
        None => None,
    };

    // Second pass, all other entities other than players
    for (i, line) in line_strings.iter().enumerate() {
        for (j, tile_char) in line.chars().enumerate() {
            let coords = file_to_tile_coords(i, j, height);
            if "fFbBtT".contains(tile_char) {
                commands.spawn_bundle(WallBundle::new(coords, &sprite_handles));
            } else if "wW".contains(tile_char) {
                commands.spawn_bundle(InputBlockBundle::new(
                    Direction::Up,
                    coords,
                    &sprite_handles,
                ));
            } else if "aA".contains(tile_char) {
                commands.spawn_bundle(InputBlockBundle::new(
                    Direction::Left,
                    coords,
                    &sprite_handles,
                ));
            } else if "sS".contains(tile_char) {
                commands.spawn_bundle(InputBlockBundle::new(
                    Direction::Down,
                    coords,
                    &sprite_handles,
                ));
            } else if "dD".contains(tile_char) {
                commands.spawn_bundle(InputBlockBundle::new(
                    Direction::Right,
                    coords,
                    &sprite_handles,
                ));
            } else if "gG".contains(tile_char) {
                commands.spawn_bundle(GoalBundle::new(coords, &sprite_handles));
            } else if tile_char == 'i' {
                commands.spawn_bundle(MoveTableBundle::new(
                    willow_id.expect("Willow table exists, but not Willow"),
                    coords,
                    &sprite_handles,
                ));
            } else if tile_char == 'c' {
                commands.spawn_bundle(MoveTableBundle::new(
                    chester_id.expect("Chester table exists, but not Chester"),
                    coords,
                    &sprite_handles,
                ));
            }
        }
    }
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
