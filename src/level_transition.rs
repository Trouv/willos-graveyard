//! Plugin, events, and utilities providing functionality for level transitions.
use crate::{
    event_scheduler::{EventScheduler, EventSchedulerPlugin},
    gameplay::components::*,
    nine_slice::*,
    ui::font_scale::{FontScale, FontSize},
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::{ldtk::FieldInstance, prelude::*};
use iyes_loopless::prelude::*;
use std::time::Duration;

/// Plugin providing functionality for level transitions.
pub struct LevelTransitionPlugin;

impl Plugin for LevelTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EventSchedulerPlugin::<LevelCardEvent>::new())
            .add_system(
                trigger_level_transition_state
                    .run_not_in_state(GameState::AssetLoading)
                    .run_not_in_state(GameState::LevelTransition)
                    .run_if_resource_added::<TransitionTo>(),
            )
            .add_exit_system(GameState::LevelTransition, clean_up_transition_to_resource)
            .add_startup_system(schedule_first_level_card)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::LevelTransition)
                    .with_system(spawn_level_card)
                    .with_system(load_next_level)
                    .with_system(level_card_update)
                    .into(),
            );
    }
}

/// Resource that can be inserted to trigger a level transition.
pub struct TransitionTo(pub LevelSelection);

/// Component that marks the level card and stores its current state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
enum LevelCard {
    Rising,
    Holding,
    Falling,
    End,
}

/// Event that fires during the level card rising/falling animation, describing the current stage
/// of the animation.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum LevelCardEvent {
    Rise(LevelSelection),
    Block(LevelSelection),
    Fall,
    Despawn,
}

/// Utility for causing a level card to perform its animation.
///
/// Must be accompanied by a change to the [GameState::LevelTransition] state to work.
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

fn trigger_level_transition_state(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::LevelTransition));
}

fn clean_up_transition_to_resource(mut commands: Commands) {
    commands.remove_resource::<TransitionTo>()
}

// TODO: change the logic for level cards to rise on entering `LevelTransition`, with some stage to
// add buffer to it if needed.
//
// Loading the first level bugs out sometimes because this is a startup system.
fn schedule_first_level_card(
    mut level_card_events: ResMut<EventScheduler<LevelCardEvent>>,
    level_selection: Res<LevelSelection>,
) {
    schedule_level_card(&mut level_card_events, level_selection.clone(), 800);
}

fn load_next_level(
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

fn spawn_level_card(
    mut commands: Commands,
    mut reader: EventReader<LevelCardEvent>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    assets: Res<AssetServer>,
    asset_holder: Res<AssetHolder>,
    mut images: ResMut<Assets<Image>>,
    ui_root_query: Query<Entity, With<UiRoot>>,
) {
    for event in reader.iter() {
        if let LevelCardEvent::Rise(level_selection) = event {
            let mut title = "Thank you for playing!\n\nMade by Trevor Lovell and Gabe Machado\n\nWayfarer's Toy Box font by Chequered Ink".to_string();
            let mut level_num = None;

            if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
                if let Some((level_index, level)) = ldtk_asset
                    .iter_levels()
                    .enumerate()
                    .find(|(i, level)| level_selection.is_match(i, level))
                {
                    if level_index < ldtk_asset.project.levels.len() {
                        level_num = Some(level_index + 1);

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

            let level_card_atlas = texture_atlas_from_nine_slice(
                asset_holder.tarot_sheet.clone(),
                Vec2::splat(64.),
                16.,
                16.,
                16.,
                16.,
            );
            let level_card_texture = generate_nineslice_image(
                NineSliceSize {
                    inner_width: 8,
                    inner_height: 4,
                },
                NineSliceIndex::default(),
                &level_card_atlas,
                &mut images,
            )
            .unwrap();

            let level_card_entity = commands
                .spawn_bundle(ImageBundle {
                    image: UiImage(level_card_texture),
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
                        parent
                            .spawn_bundle(TextBundle {
                                text: Text::from_section(
                                    format!("#{level_num}"),
                                    TextStyle {
                                        font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                )
                                .with_alignment(TextAlignment {
                                    vertical: VerticalAlign::Center,
                                    horizontal: HorizontalAlign::Center,
                                }),
                                visibility: Visibility { is_visible: false },
                                ..Default::default()
                            })
                            .insert(FontScale::from(FontSize::Huge));
                    }
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::from_section(
                                title,
                                TextStyle {
                                    font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            visibility: Visibility { is_visible: false },
                            ..Default::default()
                        })
                        .insert(FontScale::from(FontSize::Medium));
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

fn level_card_update(
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
