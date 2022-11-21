//! Plugin providing functionality for level transitions.
use crate::{
    event_scheduler::{EventScheduler, EventSchedulerPlugin},
    nine_slice::{
        generate_nineslice_image, texture_atlas_from_nine_slice, NineSliceIndex, NineSliceSize,
    },
    ui::{
        font_scale::{FontScale, FontSize},
        UiRoot,
    },
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
        app.init_resource::<TransitionTo>()
            .add_plugin(EventSchedulerPlugin::<LevelCardEvent>::new())
            .add_system(
                trigger_level_transition_state
                    .run_not_in_state(GameState::AssetLoading)
                    .run_not_in_state(GameState::LevelTransition)
                    .run_if_resource_added::<TransitionTo>(),
            )
            .add_exit_system(GameState::LevelTransition, clean_up_transition_to_resource)
            .add_enter_system(GameState::LevelTransition, spawn_level_card)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::LevelTransition)
                    .run_on_event::<LevelCardEvent>()
                    .with_system(level_card_update)
                    .with_system(load_next_level)
                    .into(),
            )
            // level_card_update should be performed during both gameplay and level transition
            // states since it cleans up the level card after it's done falling during the gameplay
            // state
            .add_system(
                level_card_update
                    .run_in_state(GameState::Gameplay)
                    .run_on_event::<LevelCardEvent>(),
            );
    }
}

/// Resource that can be inserted to trigger a level transition.
#[derive(Clone, Eq, PartialEq, Debug, Default, Deref, DerefMut, Component)]
pub struct TransitionTo(pub LevelSelection);

/// Component that marks the level card.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
struct LevelCard;

/// Event that fires during the level card rising/falling animation, describing the current stage
/// of the animation.
#[derive(Clone, Eq, PartialEq, Debug)]
enum LevelCardEvent {
    Block,
    Fall,
    Despawn,
}

fn schedule_level_card(level_card_events: &mut EventScheduler<LevelCardEvent>) {
    level_card_events.schedule(LevelCardEvent::Block, Duration::from_millis(1500));
    level_card_events.schedule(LevelCardEvent::Fall, Duration::from_millis(3000));
    level_card_events.schedule(LevelCardEvent::Despawn, Duration::from_millis(4500));
}

fn trigger_level_transition_state(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::LevelTransition));
}

fn clean_up_transition_to_resource(mut commands: Commands) {
    commands.remove_resource::<TransitionTo>()
}

fn spawn_level_card(
    mut commands: Commands,
    mut level_card_events: ResMut<EventScheduler<LevelCardEvent>>,
    transition_to: Res<TransitionTo>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    assets: Res<AssetServer>,
    asset_holder: Res<AssetHolder>,
    mut images: ResMut<Assets<Image>>,
    ui_root_query: Query<Entity, With<UiRoot>>,
) {
    let mut title = "Thank you for playing!\n\nMade by Trevor Lovell and Gabe Machado\n\nWayfarer's Toy Box font by Chequered Ink".to_string();
    let mut level_num = None;

    if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
        if let Some((level_index, level)) = ldtk_asset
            .iter_levels()
            .enumerate()
            .find(|(i, level)| transition_to.is_match(i, level))
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
        .spawn(ImageBundle {
            image: UiImage(level_card_texture),
            ..Default::default()
        })
        .insert(
            Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
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
                    flex_direction: FlexDirection::Column,
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
                    .spawn(TextBundle {
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
                        ..Default::default()
                    })
                    .insert(FontScale::from(FontSize::Huge));
            }
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        title,
                        TextStyle {
                            font: assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf"),
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..Default::default()
                })
                .insert(FontScale::from(FontSize::Medium));
        })
        .insert(LevelCard)
        .id();

    if level_num.is_some() {
        schedule_level_card(&mut level_card_events);
    }

    commands
        .entity(ui_root_query.single())
        .add_child(level_card_entity);
}

fn load_next_level(
    mut commands: Commands,
    mut level_card_events: EventReader<LevelCardEvent>,
    mut level_selection: ResMut<LevelSelection>,
    mut first_card_skipped: Local<bool>,
    transition_to: Res<TransitionTo>,
    asset_holder: Res<AssetHolder>,
) {
    for event in level_card_events.iter() {
        if let LevelCardEvent::Block = event {
            if *first_card_skipped {
                *level_selection = transition_to.0.clone()
            } else {
                commands.spawn(LdtkWorldBundle {
                    ldtk_handle: asset_holder.ldtk.clone(),
                    transform: Transform::from_xyz(32., 32., 0.),
                    ..Default::default()
                });

                *first_card_skipped = true;
            }
        }
    }
}

fn level_card_update(
    mut commands: Commands,
    mut card_query: Query<(Entity, &mut Style), With<LevelCard>>,
    mut level_card_events: EventReader<LevelCardEvent>,
) {
    for event in level_card_events.iter() {
        for (entity, style) in card_query.iter_mut() {
            match event {
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
                }
                LevelCardEvent::Despawn => {
                    // SELF DESTRUCT
                    commands.entity(entity).despawn_recursive();
                }
                _ => (),
            }
        }
    }
}
