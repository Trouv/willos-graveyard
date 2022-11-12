use crate::{
    gameplay::components::*,
    resources::*,
    sugar::GoalGhostAnimation,
    ui::font_scale::{FontScale, FontSize},
    willo::WilloState,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use std::time::Duration;

pub fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(OrthographicCamera);
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

pub fn spawn_death_card(
    mut commands: Commands,
    assets: Res<AssetServer>,
    willo_query: Query<&WilloState, Changed<WilloState>>,
    death_cards: Query<Entity, With<DeathCard>>,
    mut last_state: Local<WilloState>,
    ui_root_query: Query<Entity, With<UiRoot>>,
) {
    for state in willo_query.iter() {
        if *state == WilloState::Dead && *last_state != WilloState::Dead {
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
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::from_section(
                                "EXORCISED\n\nR to restart\nZ to undo",
                                TextStyle {
                                    font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                                    color: Color::WHITE,
                                    ..default()
                                },
                            )
                            .with_alignment(TextAlignment {
                                horizontal: HorizontalAlign::Center,
                                vertical: VerticalAlign::Center,
                            }),
                            visibility: Visibility { is_visible: false },
                            ..Default::default()
                        })
                        .insert(FontScale::from(FontSize::Medium));
                })
                .id();

            commands
                .entity(ui_root_query.single())
                .add_child(death_card_entity);
        } else if *state != WilloState::Dead && *last_state == WilloState::Dead {
            // Player just un-died
            if let Ok(entity) = death_cards.get_single() {
                commands.entity(entity).despawn_recursive();
            }
        }

        *last_state = *state;
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
            let aspect_ratio = window.width() / window.height();
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

                projection.right = pixel_perfect_width;
                projection.top = (pixel_perfect_width / aspect_ratio).round();
            } else {
                // Play zone is "tall"

                let pixel_perfect_height =
                    ((play_zone_size.height / aspect_ratio).round() * aspect_ratio).round();

                projection.right = (pixel_perfect_height * aspect_ratio).round();
                projection.top = pixel_perfect_height;
            };

            transform.translation.x =
                ((play_zone_size.width - padded_level_size.x as f32) / -2.).round();
            transform.translation.y =
                ((play_zone_size.height - padded_level_size.y as f32) / -2.).round();
        }
    }
}
