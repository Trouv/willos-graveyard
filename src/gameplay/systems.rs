use crate::{
    gameplay::{
        components::*, xy_translation, ActionEvent, Direction, LevelCardEvent, PlayerMovementEvent,
        DIRECTION_ORDER,
    },
    LevelState, SoundEffects,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use rand::Rng;
use std::cmp;

pub fn ease_movement(
    mut commands: Commands,
    mut grid_coords_query: Query<
        (Entity, &GridCoords, &Transform),
        (Changed<GridCoords>, Without<MoveTable>),
    >,
) {
    for (entity, &grid_coords, transform) in grid_coords_query.iter_mut() {
        let xy = xy_translation(grid_coords.into());

        commands.entity(entity).insert(transform.ease_to(
            Transform::from_xyz(xy.x, xy.y, transform.translation.z),
            EaseFunction::CubicOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(110),
            },
        ));
    }
}

fn push_grid_coords_recursively(
    collision_map: Vec<Vec<Option<(Entity, RigidBody)>>>,
    pusher_coords: IVec2,
    direction: Direction,
) -> Vec<Entity> {
    let pusher = collision_map[pusher_coords.y as usize][pusher_coords.x as usize]
        .expect("pusher should exist")
        .0;
    let destination = pusher_coords + IVec2::from(direction);
    if destination.x < 0
        || destination.y < 0
        || destination.y as usize >= collision_map.len()
        || destination.x as usize >= collision_map[0].len()
    {
        return Vec::new();
    }
    match collision_map[destination.y as usize][destination.x as usize] {
        None => vec![pusher],
        Some((_, RigidBody::Static)) => Vec::new(),
        Some((_, RigidBody::Dynamic)) => {
            let mut pushed_entities =
                push_grid_coords_recursively(collision_map, destination, direction);
            if pushed_entities.is_empty() {
                Vec::new()
            } else {
                pushed_entities.push(pusher);
                pushed_entities
            }
        }
    }
}

pub fn perform_grid_coords_movement(
    mut grid_coords_query: Query<(Entity, &mut GridCoords, &RigidBody, Option<&PlayerState>)>,
    mut reader: EventReader<PlayerMovementEvent>,
    level_query: Query<&Handle<LdtkLevel>>,
    levels: Res<Assets<LdtkLevel>>,
    level_state: Res<LevelState>,
    audio: Res<Audio>,
    sfx: Res<SoundEffects>,
) {
    if *level_state == LevelState::Gameplay {
        for movement_event in reader.iter() {
            let level = levels
                .get(level_query.single())
                .expect("Level should be loaded in gameplay state");

            let LayerInstance {
                c_wid: width,
                c_hei: height,
                ..
            } = level
                .level
                .layer_instances
                .clone()
                .expect("Loaded level should have layers")[0];

            let mut collision_map: Vec<Vec<Option<(Entity, RigidBody)>>> =
                vec![vec![None; width as usize]; height as usize];
            for (entity, grid_coords, rigid_body, _) in grid_coords_query.iter_mut() {
                collision_map[grid_coords.y as usize][grid_coords.x as usize] =
                    Some((entity, *rigid_body));
            }

            let (_, &player_grid_coords, _, _) = grid_coords_query
                .iter()
                .find(|(_, _, _, player)| player.is_some())
                .unwrap();

            let pushed_entities = push_grid_coords_recursively(
                collision_map,
                player_grid_coords.into(),
                movement_event.direction,
            );

            if pushed_entities.len() > 1 {
                audio.play(sfx.push.clone_weak());
            }

            for entity in pushed_entities {
                *grid_coords_query
                    .get_component_mut::<GridCoords>(entity)
                    .expect("Pushed should have GridCoords component") +=
                    GridCoords::from(IVec2::from(movement_event.direction));
            }
        }
    }
}

pub fn move_table_update(
    mut table_query: Query<(&GridCoords, &mut MoveTable)>,
    input_block_query: Query<(&GridCoords, &InputBlock)>,
) {
    for (table_grid_coords, mut table) in table_query.iter_mut() {
        table.table = [[None; 4]; 4];
        for (input_grid_coords, input_block) in input_block_query.iter() {
            let diff = *input_grid_coords - *table_grid_coords;
            let x_index = diff.x - 1;
            let y_index = -1 - diff.y;
            if x_index >= 0 && x_index < 4 && y_index >= 0 && y_index < 4 {
                // key block is in table
                table.table[y_index as usize][x_index as usize] = Some(input_block.key_code);
            }
        }
    }
}

pub fn player_state_input(mut player_query: Query<&mut PlayerState>, input: Res<Input<KeyCode>>) {
    for mut player in player_query.iter_mut() {
        if *player == PlayerState::Waiting {
            if input.just_pressed(KeyCode::W) {
                *player = PlayerState::RankMove(KeyCode::W)
            } else if input.just_pressed(KeyCode::A) {
                *player = PlayerState::RankMove(KeyCode::A)
            } else if input.just_pressed(KeyCode::S) {
                *player = PlayerState::RankMove(KeyCode::S)
            } else if input.just_pressed(KeyCode::D) {
                *player = PlayerState::RankMove(KeyCode::D)
            }
        }
    }
}

pub fn move_player_by_table(
    table_query: Query<&MoveTable>,
    mut player_query: Query<(&mut MovementTimer, &mut PlayerState)>,
    mut movement_writer: EventWriter<PlayerMovementEvent>,
    mut action_writer: EventWriter<ActionEvent>,
    time: Res<Time>,
) {
    for table in table_query.iter() {
        let (mut timer, mut player) = player_query.single_mut();
        timer.0.tick(time.delta());

        if timer.0.finished() {
            match *player {
                PlayerState::RankMove(key) => {
                    action_writer.send(ActionEvent);
                    for (i, rank) in table.table.iter().enumerate() {
                        if rank.contains(&Some(key)) {
                            movement_writer.send(PlayerMovementEvent {
                                direction: DIRECTION_ORDER[i],
                            });
                        }
                    }
                    *player = PlayerState::FileMove(key);
                    timer.0.reset();
                }
                PlayerState::FileMove(key) => {
                    for rank in table.table.iter() {
                        for (i, cell) in rank.iter().enumerate() {
                            if *cell == Some(key) {
                                movement_writer.send(PlayerMovementEvent {
                                    direction: DIRECTION_ORDER[i],
                                });
                            }
                        }
                    }
                    *player = PlayerState::Waiting;
                    timer.0.reset();
                }
                _ => {}
            }
        }
    }
}

pub fn store_current_position(
    mut reader: EventReader<ActionEvent>,
    mut objects_query: Query<(&mut History, &GridCoords)>,
) {
    for _ in reader.iter() {
        for (mut history, grid_coords) in objects_query.iter_mut() {
            history.tiles.push(*grid_coords);
        }
    }
}

pub fn rewind(
    mut player_query: Query<&mut PlayerState>,
    input: Res<Input<KeyCode>>,
    mut objects_query: Query<(&mut History, &mut GridCoords)>,
    audio: Res<Audio>,
    sfx: Res<SoundEffects>,
) {
    if let Ok(PlayerState::Waiting | PlayerState::Dead) = player_query.get_single() {
        if input.just_pressed(KeyCode::Z) {
            for (mut history, mut grid_coords) in objects_query.iter_mut() {
                if let Some(prev_state) = history.tiles.pop() {
                    *grid_coords = prev_state;
                    *player_query.single_mut() = PlayerState::Waiting;
                    audio.play(sfx.undo.clone_weak());
                }
            }
        }
    }
}

pub fn reset(
    mut player_query: Query<&mut PlayerState>,
    input: Res<Input<KeyCode>>,
    mut objects_query: Query<(&mut History, &mut GridCoords)>,
    audio: Res<Audio>,
    sfx: Res<SoundEffects>,
) {
    if let Ok(PlayerState::Waiting | PlayerState::Dead) = player_query.get_single() {
        if input.just_pressed(KeyCode::R) {
            for (mut history, mut grid_coords) in objects_query.iter_mut() {
                if let Some(initial_state) = history.tiles.get(0) {
                    *grid_coords = *initial_state;
                    history.tiles = Vec::new();
                    *player_query.single_mut() = PlayerState::Waiting;
                    audio.play(sfx.undo.clone_weak());
                }
            }
        }
    }
}

pub fn check_death(
    mut player_query: Query<(&GridCoords, &mut PlayerState)>,
    exorcism_query: Query<&GridCoords, With<ExorcismBlock>>,
    level_state: Res<LevelState>,
) {
    if *level_state == LevelState::Gameplay {
        let (player_coords, mut player_state) = player_query.single_mut();
        if *player_state != PlayerState::Dead {
            if exorcism_query
                .iter()
                .find(|&e| e == player_coords)
                .is_some()
            {
                *player_state = PlayerState::Dead;
            }
        }
    }
}

pub fn check_goal(
    goal_query: Query<&GridCoords, With<Goal>>,
    block_query: Query<&GridCoords, With<InputBlock>>,
    mut writer: EventWriter<LevelCardEvent>,
    mut level_state: ResMut<LevelState>,
    level_selection: ResMut<LevelSelection>,
    audio: Res<Audio>,
    sfx: Res<SoundEffects>,
) {
    if *level_state == LevelState::Gameplay {
        for goal_grid_coords in goal_query.iter() {
            let mut goal_met = false;
            for block_grid_coords in block_query.iter() {
                if goal_grid_coords == block_grid_coords {
                    goal_met = true;
                    break;
                }
            }
            if !goal_met {
                return ();
            }
        }

        *level_state = LevelState::Inbetween;
        if let LevelSelection::Index(num) = level_selection.into_inner() {
            *num += 1;
        }
        writer.send(LevelCardEvent::Rise);
        audio.play(sfx.victory.clone_weak());
    }
}

pub fn animate_grass_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut WindTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let mut rng = rand::thread_rng();
            let chance = rng.gen::<f32>();
            if chance <= 0.2 {
                sprite.index = cmp::min(sprite.index + 1, texture_atlas.len() - 1);
            } else if chance > 0.2 && chance <= 0.6 {
                sprite.index = cmp::max(sprite.index as i32 - 1, 0) as usize;
            }
        }
    }
}

pub fn update_control_display(
    mut commands: Commands,
    move_table_query: Query<&MoveTable, Changed<MoveTable>>,
    control_display_query: Query<Entity, With<ControlDisplayNode>>,
    assets: Res<AssetServer>,
) {
    for move_table in move_table_query.iter() {
        let control_display_entity = control_display_query.single();

        commands
            .entity(control_display_entity)
            .despawn_descendants();

        for (i, rank) in move_table.table.iter().enumerate() {
            for (j, key) in rank.iter().enumerate() {
                if let Some(key) = key {
                    let rule = (key, DIRECTION_ORDER[i], DIRECTION_ORDER[j]);

                    commands
                        .entity(control_display_entity)
                        .with_children(|parent| {
                            parent.spawn_bundle(TextBundle {
                                text: Text::with_section(
                                    format!("{rule:?}"),
                                    TextStyle {
                                        font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                                        font_size: 30.,
                                        color: Color::WHITE,
                                    },
                                    TextAlignment::default(),
                                ),
                                transform: Transform::from_xyz(0., 0., 1.),
                                ..Default::default()
                            });
                        });
                }
            }
        }
    }
}
