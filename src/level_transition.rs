//! Plugin providing functionality for level transitions.
use crate::{
    event_scheduler::{EventScheduler, EventSchedulerPlugin},
    nine_slice::{
        generate_nineslice_image, texture_atlas_from_nine_slice, NineSliceIndex, NineSliceSize,
    },
    ui::font_scale::{FontScale, FontSize},
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_easings::{Ease, EaseFunction, *};
use bevy_ecs_ldtk::prelude::*;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq, Hash, SystemSet)]
enum LevelTransitionSystemSet {
    OnLevelCardEvent,
}

/// Plugin providing functionality for level transitions.
pub struct LevelTransitionPlugin;

impl Plugin for LevelTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransitionTo>()
            .add_plugins(EventSchedulerPlugin::<LevelCardEvent>::new())
            .add_systems(
                Update,
                trigger_level_transition_state
                    .run_if(not(in_state(GameState::AssetLoading)))
                    .run_if(not(in_state(GameState::LevelTransition)))
                    .run_if(resource_added::<TransitionTo>),
            )
            .add_systems(
                OnExit(GameState::LevelTransition),
                clean_up_transition_to_resource,
            )
            .add_systems(OnEnter(GameState::LevelTransition), spawn_level_card)
            .add_systems(
                Update,
                (level_card_update, load_next_level)
                    .in_set(LevelTransitionSystemSet::OnLevelCardEvent),
            )
            .configure_sets(
                Update,
                LevelTransitionSystemSet::OnLevelCardEvent
                    .run_if(in_state(GameState::LevelTransition))
                    .run_if(on_event::<LevelCardEvent>),
            )
            // level_card_update should be performed during both graveyard and level transition
            // states since it cleans up the level card after it's done falling during the graveyard
            // state
            .add_systems(
                Update,
                level_card_update
                    .run_if(in_state(GameState::Graveyard))
                    .run_if(on_event::<LevelCardEvent>),
            );
    }
}

/// Resource that can be inserted to trigger a level transition.
#[derive(Clone, Eq, PartialEq, Debug, Default, Deref, DerefMut, Resource)]
pub struct TransitionTo(pub LevelSelection);

/// Component that marks the level card.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
struct LevelCard;

/// Event that fires during the level card rising/falling animation, describing the current stage
/// of the animation.
#[derive(Clone, Eq, PartialEq, Debug, Event)]
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

fn trigger_level_transition_state(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::LevelTransition);
}

fn clean_up_transition_to_resource(mut commands: Commands) {
    commands.remove_resource::<TransitionTo>()
}

fn spawn_level_card(
    mut commands: Commands,
    mut level_card_events: ResMut<EventScheduler<LevelCardEvent>>,
    transition_to: Res<TransitionTo>,
    ldtk_assets: Res<Assets<LdtkProject>>,
    assets: Res<AssetServer>,
    asset_holder: Res<AssetHolder>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut title = "Thank you for playing!\n\nMade by Trevor Lovell and Gabe Machado\n\nWayfarer's Toy Box font by Chequered Ink".to_string();
    let mut level_num = None;

    if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
        if let Some(selected_level) = ldtk_asset.find_raw_level_by_level_selection(&transition_to) {
            level_num = Some(
                ldtk_asset
                    .get_level_metadata_by_iid(&selected_level.iid)
                    .expect("level metadata should exist for all levels")
                    .indices()
                    .level,
            );
            title.clone_from(
                selected_level
                    .get_string_field("Title")
                    .expect("all levels should have titles"),
            );
        }
    }

    let level_card_atlas = texture_atlas_from_nine_slice(UVec2::splat(64), 16, 16, 16, 16);
    let level_card_texture = generate_nineslice_image(
        NineSliceSize {
            inner_width: 8,
            inner_height: 4,
        },
        NineSliceIndex::default(),
        &level_card_atlas,
        &asset_holder.tarot_sheet,
        &mut images,
    )
    .unwrap();

    commands
        .spawn(ImageNode::new(level_card_texture))
        .insert(
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                top: Val::Percent(100.),
                left: Val::Percent(0.),
                ..Default::default()
            }
            .ease_to(
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    top: Val::Percent(0.),
                    left: Val::Percent(0.),
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
                    .spawn((
                        Text(format!("#{level_num}")),
                        TextFont::from_font(assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf")),
                        TextColor(Color::WHITE),
                    ))
                    .insert(FontScale::from(FontSize::Huge));
            }
            parent
                .spawn((
                    Text(title),
                    TextFont::from_font(assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf")),
                    TextColor(Color::WHITE),
                ))
                .insert(FontScale::from(FontSize::Medium));
        })
        .insert(LevelCard);

    if level_num.is_some() {
        schedule_level_card(&mut level_card_events);
    }
}

fn load_next_level(
    mut commands: Commands,
    mut level_card_events: EventReader<LevelCardEvent>,
    mut level_selection: ResMut<LevelSelection>,
    mut first_card_skipped: Local<bool>,
    transition_to: Res<TransitionTo>,
    asset_holder: Res<AssetHolder>,
) {
    for event in level_card_events.read() {
        if let LevelCardEvent::Block = event {
            if *first_card_skipped {
                *level_selection = transition_to.0.clone()
            } else {
                commands.spawn(LdtkWorldBundle {
                    ldtk_handle: asset_holder.ldtk.clone().into(),
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
    mut next_state: ResMut<NextState<GameState>>,
    mut card_query: Query<(Entity, &mut Node), With<LevelCard>>,
    mut level_card_events: EventReader<LevelCardEvent>,
) {
    for event in level_card_events.read() {
        for (entity, style) in card_query.iter_mut() {
            match event {
                LevelCardEvent::Fall => {
                    commands.entity(entity).insert(style.clone().ease_to(
                        Node {
                            top: Val::Percent(100.),
                            left: Val::Percent(0.),
                            ..style.clone()
                        },
                        EaseFunction::QuadraticIn,
                        EasingType::Once {
                            duration: Duration::from_secs(1),
                        },
                    ));

                    next_state.set(GameState::Graveyard);
                }
                LevelCardEvent::Despawn => {
                    // SELF DESTRUCT
                    commands.entity(entity).despawn();
                }
                _ => (),
            }
        }
    }
}
