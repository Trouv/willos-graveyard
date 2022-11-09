use crate::{
    event_scheduler::EventScheduler,
    gameplay::components::*,
    gameplay::{DeathEvent, Direction, GoalEvent, LevelCardEvent, DIRECTION_ORDER},
    ui::font_scale::{FontScale, FontSize},
    willo::WilloState,
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

pub fn check_death(
    mut willo_query: Query<(Entity, &GridCoords, &mut WilloState)>,
    exorcism_query: Query<(Entity, &GridCoords), With<ExorcismBlock>>,
    mut death_event_writer: EventWriter<DeathEvent>,
) {
    if let Ok((entity, coords, mut willo)) = willo_query.get_single_mut() {
        if *willo != WilloState::Dead && exorcism_query.iter().any(|(_, g)| *g == *coords) {
            *willo = WilloState::Dead;
            death_event_writer.send(DeathEvent {
                willo_entity: entity,
            });
        }
    }
}

pub fn schedule_level_card(
    level_card_events: &mut EventScheduler<LevelCardEvent>,
    level_selection: LevelSelection,
    offset_millis: u64,
) {
    level_card_events.schedule(
        LevelCardEvent::Rise(level_selection.clone()),
        Duration::from_millis(offset_millis),
    );
    level_card_events.schedule(
        LevelCardEvent::Block(level_selection),
        Duration::from_millis(1500 + offset_millis),
    );
    level_card_events.schedule(
        LevelCardEvent::Fall,
        Duration::from_millis(3000 + offset_millis),
    );
    level_card_events.schedule(
        LevelCardEvent::Despawn,
        Duration::from_millis(4500 + offset_millis),
    );
}

pub fn check_goal(
    mut commands: Commands,
    mut goal_query: Query<(Entity, &mut Goal, &GridCoords), With<Goal>>,
    block_query: Query<(Entity, &GridCoords), With<InputBlock>>,
    mut level_card_events: ResMut<EventScheduler<LevelCardEvent>>,
    mut goal_events: EventWriter<GoalEvent>,
    level_selection: Res<LevelSelection>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    audio: Res<Audio>,
    asset_holder: Res<AssetHolder>,
) {
    // If the goal is not loaded for whatever reason (for example when hot-reloading levels),
    // the goal will automatically be "met", loading the next level.
    // This if statement prevents that.
    if goal_query.iter().count() == 0 {
        return;
    }

    let mut level_goal_met = true;

    for (goal_entity, mut goal, goal_grid_coords) in goal_query.iter_mut() {
        let mut goal_met = false;
        for (stone_entity, block_grid_coords) in block_query.iter() {
            if goal_grid_coords == block_grid_coords {
                goal_met = true;

                if !goal.met {
                    goal.met = true;

                    goal_events.send(GoalEvent::Met {
                        stone_entity,
                        goal_entity,
                    });
                }

                break;
            }
        }
        if !goal_met {
            level_goal_met = false;

            if goal.met {
                goal_events.send(GoalEvent::UnMet { goal_entity });
                goal.met = false;
            }
        }
    }

    if level_goal_met {
        commands.insert_resource(NextState(GameState::LevelTransition));

        if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
            if let Some((level_index, _)) = ldtk_asset
                .iter_levels()
                .enumerate()
                .find(|(i, level)| level_selection.is_match(i, level))
            {
                schedule_level_card(
                    &mut level_card_events,
                    LevelSelection::Index(level_index + 1),
                    800,
                );
            }
        }

        audio.play(asset_holder.victory_sound.clone_weak());
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
            color: Color::WHITE,
            ..default()
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
                                margin: UiRect {
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
                                        parent
                                            .spawn_bundle(TextBundle {
                                                text: Text::from_section(s, style.clone())
                                                    .with_alignment(TextAlignment {
                                                        vertical: VerticalAlign::Center,
                                                        horizontal: HorizontalAlign::Center,
                                                    }),
                                                style: Style {
                                                    size: Size {
                                                        height: Val::Percent(100.),
                                                        ..Default::default()
                                                    },
                                                    margin: UiRect {
                                                        right: Val::Px(16.),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            })
                                            .insert(FontScale::from(FontSize::Medium));
                                    }
                                    ControlNode::Image(h) => {
                                        parent.spawn_bundle(ImageBundle {
                                            image: UiImage(h),
                                            style: Style {
                                                size: Size {
                                                    height: Val::Percent(100.),
                                                    ..Default::default()
                                                },
                                                margin: UiRect {
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
