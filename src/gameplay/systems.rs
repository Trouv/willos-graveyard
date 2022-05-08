use crate::{
    event_scheduler::EventScheduler,
    gameplay::{
        components::*, DeathEvent, Direction, LevelCardEvent, PlayerMovementEvent, DIRECTION_ORDER,
    },
    history::HistoryCommands,
    sugar::PlayerAnimationState,
    LevelState, SoundEffects,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use std::time::Duration;

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
    mut grid_coords_query: Query<(
        Entity,
        &mut GridCoords,
        &RigidBody,
        Option<&mut PlayerAnimationState>,
    )>,
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

            if let Some((_, player_grid_coords, _, Some(mut animation))) = grid_coords_query
                .iter_mut()
                .find(|(_, _, _, animation)| animation.is_some())
            {
                let pushed_entities = push_grid_coords_recursively(
                    collision_map,
                    IVec2::from(*player_grid_coords),
                    movement_event.direction,
                );

                if pushed_entities.len() > 1 {
                    audio.play(sfx.push.clone_weak());
                    *animation.into_inner() = PlayerAnimationState::Push(movement_event.direction);
                } else {
                    let new_state = PlayerAnimationState::Idle(movement_event.direction);
                    if *animation != new_state {
                        *animation = new_state;
                    }
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
            if (0..4).contains(&x_index) && y_index >= 0 && y_index < 4 {
                // key block is in table
                table.table[y_index as usize][x_index as usize] = Some(input_block.key_code);
            }
        }
    }
}

pub fn player_state_input(
    mut player_query: Query<&mut PlayerState>,
    input: Res<Input<KeyCode>>,
    mut history_commands: EventWriter<HistoryCommands>,
) {
    for mut player in player_query.iter_mut() {
        if *player == PlayerState::Waiting {
            if input.just_pressed(KeyCode::W) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::W)
            } else if input.just_pressed(KeyCode::A) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::A)
            } else if input.just_pressed(KeyCode::S) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::S)
            } else if input.just_pressed(KeyCode::D) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::D)
            }
        }

        if *player == PlayerState::Waiting || *player == PlayerState::Dead {
            if input.just_pressed(KeyCode::Z) {
                history_commands.send(HistoryCommands::Rewind);
                *player = PlayerState::Waiting;
            } else if input.just_pressed(KeyCode::R) {
                history_commands.send(HistoryCommands::Reset);
                *player = PlayerState::Waiting;
            }
        }
    }
}

pub fn move_player_by_table(
    table_query: Query<&MoveTable>,
    mut player_query: Query<(&mut MovementTimer, &mut PlayerState)>,
    mut movement_writer: EventWriter<PlayerMovementEvent>,
    time: Res<Time>,
) {
    for table in table_query.iter() {
        if let Ok((mut timer, mut player)) = player_query.get_single_mut() {
            timer.0.tick(time.delta());

            if timer.0.finished() {
                match *player {
                    PlayerState::RankMove(key) => {
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
}

pub fn check_death(
    mut player_query: Query<(Entity, &GridCoords, &mut PlayerState)>,
    exorcism_query: Query<(Entity, &GridCoords), With<ExorcismBlock>>,
    level_state: Res<LevelState>,
    mut death_event_writer: EventWriter<DeathEvent>,
) {
    if *level_state == LevelState::Gameplay {
        if let Ok((entity, player_coords, mut player_state)) = player_query.get_single_mut() {
            if *player_state != PlayerState::Dead {
                if let Some((exorcism_entity, _)) =
                    exorcism_query.iter().find(|(_, g)| *g == player_coords)
                {
                    *player_state = PlayerState::Dead;
                    death_event_writer.send(DeathEvent {
                        player_entity: entity,
                        exorcism_entity,
                    });
                }
            }
        }
    }
}

pub fn schedule_level_card(
    level_card_events: &mut EventScheduler<LevelCardEvent>,
    level_selection: LevelSelection,
) {
    level_card_events.schedule(
        LevelCardEvent::Rise(level_selection.clone()),
        Duration::from_millis(100),
    );
    level_card_events.schedule(
        LevelCardEvent::Block(level_selection),
        Duration::from_millis(1600),
    );
    level_card_events.schedule(LevelCardEvent::Fall, Duration::from_millis(3100));
    level_card_events.schedule(LevelCardEvent::Despawn, Duration::from_millis(4600));
}

pub fn check_goal(
    goal_query: Query<&GridCoords, With<Goal>>,
    block_query: Query<&GridCoords, With<InputBlock>>,
    mut level_card_events: ResMut<EventScheduler<LevelCardEvent>>,
    mut level_state: ResMut<LevelState>,
    level_selection: Res<LevelSelection>,
    audio: Res<Audio>,
    sfx: Res<SoundEffects>,
) {
    if *level_state == LevelState::Gameplay {
        // If the goal is not loaded for whatever reason (for example when hot-reloading levels),
        // the goal will automatically be "met", loading the next level.
        // This if statement prevents that.
        if goal_query.iter().count() == 0 {
            return;
        }

        for goal_grid_coords in goal_query.iter() {
            let mut goal_met = false;
            for block_grid_coords in block_query.iter() {
                if goal_grid_coords == block_grid_coords {
                    goal_met = true;
                    break;
                }
            }
            if !goal_met {
                return;
            }
        }

        *level_state = LevelState::Inbetween;

        if let LevelSelection::Index(num) = *level_selection {
            schedule_level_card(&mut level_card_events, LevelSelection::Index(num + 1));
        }

        audio.play(sfx.victory.clone_weak());
    }
}

pub fn update_control_display(
    mut commands: Commands,
    move_table_query: Query<&MoveTable, Changed<MoveTable>>,
    control_display_query: Query<Entity, With<ControlDisplayNode>>,
    assets: Res<AssetServer>,
) {
    enum ControlNode {
        Text(String),
        Image(Handle<Image>),
    }

    for move_table in move_table_query.iter() {
        let control_display_entity = control_display_query.single();

        commands
            .entity(control_display_entity)
            .despawn_descendants();

        let font = assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf");

        let style = TextStyle {
            font,
            font_size: 30.,
            color: Color::WHITE,
        };
        commands
            .entity(control_display_entity)
            .with_children(|parent| {
                let mut add_row = |nodes: Vec<ControlNode>| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Percent(100. / 18.),
                                    ..Default::default()
                                },
                                margin: Rect {
                                    bottom: Val::Px(16.),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            color: UiColor(Color::NONE),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            for node in nodes {
                                match node {
                                    ControlNode::Text(s) => {
                                        parent.spawn_bundle(TextBundle {
                                            text: Text::with_section(
                                                s,
                                                style.clone(),
                                                TextAlignment {
                                                    vertical: VerticalAlign::Center,
                                                    horizontal: HorizontalAlign::Center,
                                                },
                                            ),
                                            style: Style {
                                                size: Size {
                                                    height: Val::Percent(100.),
                                                    ..Default::default()
                                                },
                                                margin: Rect {
                                                    right: Val::Px(16.),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        });
                                    }
                                    ControlNode::Image(h) => {
                                        parent.spawn_bundle(ImageBundle {
                                            image: UiImage(h),
                                            style: Style {
                                                size: Size {
                                                    height: Val::Percent(100.),
                                                    ..Default::default()
                                                },
                                                margin: Rect {
                                                    right: Val::Px(16.),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        });
                };

                let mut keys_to_controls: Vec<(KeyCode, Vec<ControlNode>)> = vec![
                    (
                        KeyCode::W,
                        vec![
                            ControlNode::Image(assets.load("textures/w.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                    (
                        KeyCode::A,
                        vec![
                            ControlNode::Image(assets.load("textures/a.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                    (
                        KeyCode::S,
                        vec![
                            ControlNode::Image(assets.load("textures/s.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                    (
                        KeyCode::D,
                        vec![
                            ControlNode::Image(assets.load("textures/d.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                ];

                for (i, rank) in move_table.table.iter().enumerate() {
                    for (j, key) in rank.iter().enumerate() {
                        if let Some(key) = key {
                            let first_dir = DIRECTION_ORDER[i];
                            let second_dir = DIRECTION_ORDER[j];

                            let direction_handle = |d: Direction| -> Handle<Image> {
                                match d {
                                    Direction::Up => assets.load("textures/up.png"),
                                    Direction::Left => assets.load("textures/left.png"),
                                    Direction::Down => assets.load("textures/down.png"),
                                    Direction::Right => assets.load("textures/right.png"),
                                }
                            };

                            if let Some((_, controls)) =
                                keys_to_controls.iter_mut().find(|(k, _)| k == key)
                            {
                                controls.extend(vec![
                                    ControlNode::Image(direction_handle(first_dir)),
                                    ControlNode::Image(direction_handle(second_dir)),
                                ]);
                            }
                        }
                    }
                }

                keys_to_controls
                    .into_iter()
                    .for_each(|(_, row)| add_row(row));

                add_row(vec![
                    ControlNode::Text("R".to_string()),
                    ControlNode::Text("=".to_string()),
                    ControlNode::Text("restart".to_string()),
                ]);
                add_row(vec![
                    ControlNode::Text("Z".to_string()),
                    ControlNode::Text("=".to_string()),
                    ControlNode::Text("undo".to_string()),
                ]);
            });
    }
}
