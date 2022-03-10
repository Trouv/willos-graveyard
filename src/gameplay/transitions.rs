use crate::{
    gameplay::{
        bundles::*, components::*, CardUpEvent, Direction, LevelCompleteEvent, LevelStartEvent,
        DIRECTION_ORDER,
    },
    utils::application_root_dir,
    LevelEntities, LevelNum, LevelSize, LevelState, SpriteHandles, LEVEL_ORDER, UNIT_LENGTH,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    time::Duration,
};

use super::xy_translation;

fn file_to_tile_coords(i: usize, j: usize, height: usize) -> IVec2 {
    IVec2::new(j as i32, height as i32 - i as i32 - 1)
}

fn get_level_title_data(level_num: &LevelNum) -> (String, Vec<String>) {
    let level_path = application_root_dir()
        .unwrap()
        .join(Path::new("assets/levels/"))
        .join(Path::new(LEVEL_ORDER[level_num.0]));

    let mut lines =
        BufReader::new(File::open(level_path).expect("level file should exist")).lines();
    let title = lines.next().unwrap().unwrap();
    (title, lines.map(|x| x.unwrap()).collect::<Vec<String>>())
}

pub fn world_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(OrthographicCamera);

    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/sokoban-sokoban.ldtk"),
        transform: Transform::from_xyz(32., 32., 0.),
        ..Default::default()
    });
}

//pub fn load_level(
//mut commands: Commands,
//sprite_handles: Res<SpriteHandles>,
//level_num: Res<LevelNum>,
//mut level_entities: ResMut<LevelEntities>,
//mut reader: EventReader<CardUpEvent>,
//) -> Option<LevelSize> {
//for _ in reader.iter() {
//// Unload last level
//while let Some(entity) = level_entities.0.pop() {
//commands.entity(entity).despawn_recursive();
//}

//let (_, line_strings) = get_level_title_data(&level_num);

//let mut willow = None;
//let mut chester = None;
//let mut width = 0;
//let mut height = 0;
//// Player pass, and get width and height
//for (i, line) in line_strings.clone().iter().enumerate() {
//for (j, tile_char) in line.chars().enumerate() {
//if tile_char == 'I' {
//willow = Some((i, j))
//} else if tile_char == 'C' {
//chester = Some((i, j))
//}
//if j + 1 > width {
//width = j + 1
//};
//}
//if i + 1 > height {
//height = i + 1
//};
//}

//let willow_id = match willow {
//Some(w) => Some(
//commands
//.spawn_bundle(PlayerBundle::new(
//file_to_tile_coords(w.0, w.1, height),
//&&sprite_handles,
//))
//.id(),
//),
//None => None,
//};
//if let Some(entity) = willow_id {
//level_entities.0.push(entity)
//}

//let chester_id = match chester {
//Some(c) => Some(
//commands
//.spawn_bundle(PlayerBundle::new(
//file_to_tile_coords(c.0, c.1, height),
//&&sprite_handles,
//))
//.id(),
//),
//None => None,
//};
//if let Some(entity) = chester_id {
//level_entities.0.push(entity)
//}

//// Second pass, all other entities other than players
//for (i, line) in line_strings.iter().enumerate() {
//for (j, tile_char) in line.chars().enumerate() {
//let coords = file_to_tile_coords(i, j, height);
//if "fF".contains(tile_char) {
//level_entities.0.push(
//commands
//.spawn_bundle(WallBundle::new(coords, &sprite_handles))
//.id(),
//);
//} else if "bBtT".contains(tile_char) {
//level_entities.0.push(
//commands
//.spawn_bundle(BushBundle::new(coords, &sprite_handles))
//.id(),
//);
//} else if "wW".contains(tile_char) {
//level_entities.0.push(
//commands
//.spawn_bundle(InputBlockBundle::new(
//Direction::Up,
//coords,
//&sprite_handles,
//))
//.id(),
//);
//} else if "aA".contains(tile_char) {
//level_entities.0.push(
//commands
//.spawn_bundle(InputBlockBundle::new(
//Direction::Left,
//coords,
//&sprite_handles,
//))
//.id(),
//);
//} else if "sS".contains(tile_char) {
//level_entities.0.push(
//commands
//.spawn_bundle(InputBlockBundle::new(
//Direction::Down,
//coords,
//&sprite_handles,
//))
//.id(),
//);
//} else if "dD".contains(tile_char) {
//level_entities.0.push(
//commands
//.spawn_bundle(InputBlockBundle::new(
//Direction::Right,
//coords,
//&sprite_handles,
//))
//.id(),
//);
//} else if "gG".contains(tile_char) {
//level_entities.0.push(
//commands
//.spawn_bundle(GoalBundle::new(coords, &sprite_handles))
//.id(),
//);
//} else if tile_char == 'i' {
//level_entities.0.push(
//commands
//.spawn_bundle(MoveTableBundle::new(
//willow_id.expect("Willow table exists, but not Willow"),
//coords,
//&sprite_handles,
//))
//.id(),
//);
//} else if tile_char == 'c' {
//level_entities.0.push(
//commands
//.spawn_bundle(MoveTableBundle::new(
//chester_id.expect("Chester table exists, but not Chester"),
//coords,
//&sprite_handles,
//))
//.id(),
//);
//}
////Create Background Grass
//let xy = xy_translation(coords);
//level_entities.0.push(
//commands
//.spawn_bundle(SpriteSheetBundle {
//texture_atlas: sprite_handles.get_rand_grass(),
//transform: Transform::from_translation(Vec3::new(xy.x, xy.y, 0.)),
//..Default::default()
//})
//.insert(Timer::from_seconds(0.1, true))
//.id(),
//);
//}
//}
//let level_size = LevelSize {
//size: IVec2::new(width as i32, height as i32),
//};
//commands.insert_resource(level_size);
//return Some(level_size);
//}
//return None;
//}

//pub fn spawn_table_edges(
//mut commands: Commands,
//table_query: Query<&Tile, Added<MoveTable>>,
//sprite_handles: Res<SpriteHandles>,
//mut level_entities: ResMut<LevelEntities>,
//) {
//for tile in table_query.iter() {
//for (i, direction) in DIRECTION_ORDER.iter().enumerate() {
//level_entities.0.push(
//commands
//.spawn_bundle(DirectionTileBundle::new(
//*direction,
//tile.coords + (i as i32 + 1) * IVec2::from(Direction::Right),
//&sprite_handles,
//))
//.id(),
//);
//level_entities.0.push(
//commands
//.spawn_bundle(DirectionTileBundle::new(
//*direction,
//tile.coords + (i as i32 + 1) * IVec2::from(Direction::Down),
//&sprite_handles,
//))
//.id(),
//);
//}
//}
//}

pub fn create_camera(
    In(level_size_input): In<Option<LevelSize>>,
    mut commands: Commands,
    mut level_entities: ResMut<LevelEntities>,
) {
    if let Some(level_size) = level_size_input {
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
        level_entities
            .0
            .push(commands.spawn().insert_bundle(camera_bundle).id());
    }
}

pub fn spawn_level_card(
    mut commands: Commands,
    mut reader: EventReader<LevelCompleteEvent>,
    level_selection: Res<LevelSelection>,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in reader.iter() {
        //let not_final = level_selection.0 < LEVEL_ORDER.len();
        let title = //if not_final {
         //   get_level_title_data(&level_selection)
        //} else 
        {
            "Thank you for playing!\n\nMade by Trevor Lovell and Gabe Machado\n\nWayfarer's Toy Box font by Chequered Ink".to_string()
        };
        commands
            .spawn_bundle(NodeBundle {
                color: UiColor(Color::BLACK),
                ..Default::default()
            })
            .insert(
                Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::ColumnReverse,
                    size: Size {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                    },
                    position: Rect {
                        top: Val::Percent(100.),
                        left: Val::Percent(0.),
                        ..Default::default()
                    },
                    ..Default::default()
                }
                .ease_to(
                    Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::ColumnReverse,
                        size: Size {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                        },
                        position: Rect {
                            top: Val::Percent(0.),
                            left: Val::Percent(0.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    EaseFunction::QuadraticOut,
                    EasingType::Once {
                        duration: Duration::from_secs(1),
                    },
                ),
            )
            .insert(Timer::new(Duration::from_millis(1500), false))
            .with_children(|parent| {
                //if not_final {
                //parent.spawn_bundle(TextBundle {
                //text: Text::with_section(
                //format!("#{}", level_selection.0),
                //TextStyle {
                //font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                //font_size: 50.,
                //color: Color::WHITE,
                //},
                //TextAlignment {
                //vertical: VerticalAlign::Center,
                //horizontal: HorizontalAlign::Center,
                //},
                //),
                //..Default::default()
                //});
                //}
                parent.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        title,
                        TextStyle {
                            font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                            font_size: 30.,
                            color: Color::WHITE,
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                });
            })
            .insert(
                //if not_final {
                LevelCard::Rising,
            );
        //} else {
        //LevelCard::End
        //});
    }
}

pub fn level_card_update(
    mut commands: Commands,
    mut card_query: Query<(Entity, &mut LevelCard, &mut Style, &mut Timer)>,
    mut card_up_writer: EventWriter<CardUpEvent>,
    mut level_start_writer: EventWriter<LevelStartEvent>,
    mut level_state: ResMut<LevelState>,
    time: Res<Time>,
) {
    for (entity, mut card, style, mut timer) in card_query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            match *card {
                LevelCard::Rising => {
                    card_up_writer.send(CardUpEvent);
                    *card = LevelCard::Holding;
                }
                LevelCard::Holding => {
                    level_start_writer.send(LevelStartEvent);

                    commands.entity(entity).insert(style.clone().ease_to(
                        Style {
                            position: Rect {
                                top: Val::Percent(100.),
                                left: Val::Percent(0.),
                                ..Default::default()
                            },
                            ..*style
                        },
                        EaseFunction::QuadraticIn,
                        EasingType::Once {
                            duration: Duration::from_secs(1),
                        },
                    ));

                    *level_state = LevelState::Gameplay;
                    *card = LevelCard::Falling;
                }
                LevelCard::Falling => {
                    // SELF DESTRUCT
                    commands.entity(entity).despawn_recursive();
                }
                _ => {}
            }
            timer.reset();
        }
    }
}

const PLAY_ZONE_RATIO: Size<i32> = Size {
    width: 4,
    height: 3,
};

const ASPECT_RATIO: Size<i32> = Size {
    width: 16,
    height: 9,
};

pub fn fit_camera_around_play_zone_padded(
    mut camera_query: Query<
        (&mut Transform, &mut OrthographicProjection),
        With<OrthographicCamera>,
    >,
    mut level_events: EventReader<LevelEvent>,
    level_query: Query<&Handle<LdtkLevel>>,
    levels: Res<Assets<LdtkLevel>>,
) {
    for level_event in level_events.iter() {
        match level_event {
            LevelEvent::Transformed(_) => {
                let level_handle = level_query.single();
                if let Some(level) = levels.get(level_handle) {
                    let level_size = IVec2::new(level.level.px_wid, level.level.px_hei);
                    let padded_level_size = level_size + IVec2::splat(32 * 2);

                    let padded_level_ratio =
                        padded_level_size.x as f32 / padded_level_size.y as f32;
                    let play_zone_ratio =
                        PLAY_ZONE_RATIO.width as f32 / PLAY_ZONE_RATIO.height as f32;
                    let aspect_ratio = ASPECT_RATIO.width as f32 / ASPECT_RATIO.height as f32;

                    let (mut transform, mut projection) = camera_query.single_mut();
                    projection.scaling_mode = bevy::render::camera::ScalingMode::None;
                    projection.bottom = 0.;
                    projection.left = 0.;

                    let play_zone_size = if padded_level_ratio > play_zone_ratio {
                        // Level is "wide"
                        Size {
                            width: padded_level_size.x as f32,
                            height: padded_level_size.x as f32 / play_zone_ratio,
                        }
                    } else {
                        // Level is "tall"
                        Size {
                            width: padded_level_size.y as f32 * play_zone_ratio,
                            height: padded_level_size.y as f32,
                        }
                    };

                    if play_zone_ratio > aspect_ratio {
                        // Play zone is "wide"
                        projection.right = play_zone_size.width;
                        projection.top = play_zone_size.width / aspect_ratio;
                    } else {
                        // Play zone is "tall"
                        projection.right = play_zone_size.height * aspect_ratio;
                        projection.top = play_zone_size.height;
                    };

                    transform.translation.x = (projection.right - padded_level_size.x as f32) / -2.;
                    transform.translation.y = (projection.top - padded_level_size.y as f32) / -2.;

                    println!("{projection:?}")
                }
            }
            _ => (),
        }
    }
}
