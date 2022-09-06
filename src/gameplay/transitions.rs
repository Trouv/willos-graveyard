use crate::{
    event_scheduler::EventScheduler,
    gameplay::{components::*, systems::schedule_level_card, LevelCardEvent},
    resources::*,
    sugar::GoalGhostAnimation,
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::{ldtk::FieldInstance, prelude::*};
use iyes_loopless::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};
use std::time::Duration;

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(OrthographicCamera);
}

pub fn spawn_gravestone_body(
    mut commands: Commands,
    gravestones: Query<(Entity, &Handle<TextureAtlas>), Added<InputBlock>>,
) {
    for (entity, texture_handle) in gravestones.iter() {
        let index_range = 11..22_usize;

        let dist: Vec<usize> = (1..(index_range.len() + 1)).map(|x| x * x).rev().collect();

        let dist = WeightedIndex::new(dist).unwrap();

        let mut rng = rand::thread_rng();

        let body_entity = commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: (11..22_usize).collect::<Vec<usize>>()[dist.sample(&mut rng)],
                    ..default()
                },
                texture_atlas: texture_handle.clone(),
                transform: Transform::from_xyz(0., 0., -0.5),
                ..default()
            })
            .id();

        commands.entity(entity).add_child(body_entity);
    }
}

pub fn spawn_goal_ghosts(
    mut commands: Commands,
    goals: Query<Entity, Added<Goal>>,
    mut goal_ghost_settings: ResMut<GoalGhostSettings>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for goal_entity in goals.iter() {
        let atlas_handle = match &goal_ghost_settings.atlas {
            Some(atlas) => atlas.clone(),
            None => {
                let image_handle = asset_server.load("textures/animations/goal_ghost-Sheet.png");
                let texture_atlas = TextureAtlas::from_grid(
                    image_handle,
                    Vec2::splat(32.),
                    goal_ghost_settings.num_columns,
                    goal_ghost_settings.num_rows,
                );
                let atlas_handle = texture_atlases.add(texture_atlas);

                goal_ghost_settings.atlas = Some(atlas_handle.clone());
                atlas_handle.clone()
            }
        };

        let ghost_entity = commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: atlas_handle,
                transform: Transform::from_xyz(0., 1., 2.5),
                ..default()
            })
            .insert(GoalGhostAnimation::new(
                goal_entity,
                Timer::new(goal_ghost_settings.frame_duration, true),
            ))
            .id();

        commands.entity(goal_entity).add_child(ghost_entity);
    }
}

pub fn spawn_ui_root(mut commands: Commands) {
    commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(UiRoot);
}

pub fn spawn_control_display(
    mut commands: Commands,
    ui_root_query: Query<Entity, Added<UiRoot>>,
    play_zone_portion: Res<PlayZonePortion>,
) {
    for ui_root_entity in ui_root_query.iter() {
        let control_zone_ratio = 1. - **play_zone_portion;

        let control_display_entity = commands
            .spawn_bundle(NodeBundle {
                color: UiColor(Color::NONE),
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    position_type: PositionType::Absolute,
                    size: Size {
                        width: Val::Percent(100. * control_zone_ratio),
                        height: Val::Percent(100.),
                    },
                    position: UiRect {
                        top: Val::Percent(0.),
                        right: Val::Percent(0.),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 0., 1.),
                ..Default::default()
            })
            .insert(ControlDisplayNode)
            .id();

        commands
            .entity(ui_root_entity)
            .add_child(control_display_entity);
    }
}

pub fn spawn_death_card(
    mut commands: Commands,
    assets: Res<AssetServer>,
    player_query: Query<&PlayerState, Changed<PlayerState>>,
    death_cards: Query<Entity, With<DeathCard>>,
    mut last_state: Local<PlayerState>,
    ui_root_query: Query<Entity, With<UiRoot>>,
) {
    for state in player_query.iter() {
        if *state == PlayerState::Dead && *last_state != PlayerState::Dead {
            // Player just died
            let death_card_entity = commands
                .spawn_bundle(NodeBundle {
                    color: UiColor(Color::rgba(0., 0., 0., 0.9)),
                    visibility: Visibility { is_visible: false },
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
                        position: UiRect {
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
                            position: UiRect {
                                top: Val::Percent(0.),
                                left: Val::Percent(0.),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        EaseFunction::QuadraticOut,
                        EasingType::Once {
                            duration: Duration::from_millis(600),
                        },
                    ),
                )
                .insert(DeathCard)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section(
                            "EXORCISED\n\nR to restart\nZ to undo",
                            TextStyle {
                                font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                                font_size: 30.,
                                color: Color::WHITE,
                            },
                        )
                        .with_alignment(TextAlignment {
                            horizontal: HorizontalAlign::Center,
                            vertical: VerticalAlign::Center,
                        }),
                        visibility: Visibility { is_visible: false },
                        ..Default::default()
                    });
                })
                .id();

            commands
                .entity(ui_root_query.single())
                .add_child(death_card_entity);
        } else if *state != PlayerState::Dead && *last_state == PlayerState::Dead {
            // Player just un-died
            if let Ok(entity) = death_cards.get_single() {
                commands.entity(entity).despawn_recursive();
            }
        }

        *last_state = *state;
    }
}

pub fn schedule_first_level_card(
    mut level_card_events: ResMut<EventScheduler<LevelCardEvent>>,
    level_selection: Res<LevelSelection>,
) {
    schedule_level_card(&mut level_card_events, level_selection.clone());
}

pub fn load_next_level(
    mut commands: Commands,
    mut level_card_events: EventReader<LevelCardEvent>,
    mut level_selection: ResMut<LevelSelection>,
    mut first_card_skipped: Local<bool>,
    asset_holder: Res<AssetHolder>,
) {
    for event in level_card_events.iter() {
        if let LevelCardEvent::Block(new_level_selection) = event {
            if *first_card_skipped {
                *level_selection = new_level_selection.clone()
            } else {
                commands.spawn_bundle(LdtkWorldBundle {
                    ldtk_handle: asset_holder.ldtk.clone(),
                    transform: Transform::from_xyz(32., 32., 0.),
                    ..Default::default()
                });

                *first_card_skipped = true;
            }
        }
    }
}

pub fn spawn_level_card(
    mut commands: Commands,
    mut reader: EventReader<LevelCardEvent>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    assets: Res<AssetServer>,
    ui_root_query: Query<Entity, With<UiRoot>>,
) {
    for event in reader.iter() {
        if let LevelCardEvent::Rise(level_selection) = event {
            let mut title = "Thank you for playing!\n\nMade by Trevor Lovell and Gabe Machado\n\nWayfarer's Toy Box font by Chequered Ink".to_string();
            let mut level_num = None;

            if let Some((_, ldtk_asset)) = ldtk_assets.iter().next() {
                if let LevelSelection::Index(level_index) = *level_selection {
                    if level_index < ldtk_asset.project.levels.len() {
                        level_num = Some(level_index + 1);

                        let level = ldtk_asset.project.levels.get(level_index).unwrap();

                        if let Some(FieldInstance {
                            value: FieldValue::String(Some(level_title)),
                            ..
                        }) = level
                            .field_instances
                            .iter()
                            .find(|f| f.identifier == "Title")
                        {
                            title = level_title.clone();
                        }
                    }
                }
            }

            let level_card_entity = commands
                .spawn_bundle(NodeBundle {
                    color: UiColor(Color::BLACK),
                    visibility: Visibility { is_visible: false },
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
                        position: UiRect {
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
                            position: UiRect {
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
                .with_children(|parent| {
                    if let Some(level_num) = level_num {
                        parent.spawn_bundle(TextBundle {
                            text: Text::from_section(
                                format!("#{}", level_num),
                                TextStyle {
                                    font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                                    font_size: 50.,
                                    color: Color::WHITE,
                                },
                            )
                            .with_alignment(TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            }),
                            visibility: Visibility { is_visible: false },
                            ..Default::default()
                        });
                    }
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section(
                            title,
                            TextStyle {
                                font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                                font_size: 30.,
                                color: Color::WHITE,
                            },
                        ),
                        visibility: Visibility { is_visible: false },
                        ..Default::default()
                    });
                })
                .insert(if level_num.is_some() {
                    LevelCard::Rising
                } else {
                    LevelCard::End
                })
                .id();

            commands
                .entity(ui_root_query.single())
                .add_child(level_card_entity);
        }
    }
}

pub fn level_card_update(
    mut commands: Commands,
    mut card_query: Query<(Entity, &mut LevelCard, &mut Style)>,
    mut level_card_events: EventReader<LevelCardEvent>,
) {
    for event in level_card_events.iter() {
        for (entity, mut card, style) in card_query.iter_mut() {
            match event {
                LevelCardEvent::Block(_) => {
                    *card = LevelCard::Holding;
                }
                LevelCardEvent::Fall => {
                    commands.entity(entity).insert(style.clone().ease_to(
                        Style {
                            position: UiRect {
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

                    commands.insert_resource(NextState(GameState::Gameplay));
                    *card = LevelCard::Falling;
                }
                LevelCardEvent::Despawn => {
                    // SELF DESTRUCT
                    commands.entity(entity).despawn_recursive();
                }
                _ => {}
            }
        }
    }
}

pub fn fit_camera_around_play_zone_padded(
    mut camera_query: Query<
        (&mut Transform, &mut OrthographicProjection),
        With<OrthographicCamera>,
    >,
    level_query: Query<&Handle<LdtkLevel>>,
    levels: Res<Assets<LdtkLevel>>,
    windows: Res<Windows>,
    play_zone_portion: Res<PlayZonePortion>,
) {
    if let Ok(level_handle) = level_query.get_single() {
        if let Some(level) = levels.get(level_handle) {
            let level_size = IVec2::new(level.level.px_wid, level.level.px_hei);
            let padded_level_size = level_size + IVec2::splat(32 * 2);

            let window = windows.primary();

            let padded_level_ratio = padded_level_size.x as f32 / padded_level_size.y as f32;
            let aspect_ratio = window.width() as f32 / window.height() as f32;
            let play_zone_ratio = aspect_ratio * **play_zone_portion;

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
                let pixel_perfect_width =
                    ((play_zone_size.width / aspect_ratio).round() * aspect_ratio).round();

                projection.right = pixel_perfect_width as f32;
                projection.top = (pixel_perfect_width as f32 / aspect_ratio).round();
            } else {
                // Play zone is "tall"

                let pixel_perfect_height =
                    ((play_zone_size.height / aspect_ratio).round() * aspect_ratio).round();

                projection.right = (pixel_perfect_height as f32 * aspect_ratio).round();
                projection.top = pixel_perfect_height as f32;
            };

            transform.translation.x =
                ((play_zone_size.width - padded_level_size.x as f32) / -2.).round();
            transform.translation.y =
                ((play_zone_size.height - padded_level_size.y as f32) / -2.).round();
        }
    }
}
