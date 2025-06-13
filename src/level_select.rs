//! Plugin providing functionality for the level select card/menu.
use crate::{
    event_scheduler::{EventScheduler, EventSchedulerPlugin},
    graveyard::GraveyardAction,
    level_transition::TransitionTo,
    nine_slice::{
        generate_nineslice_image, texture_atlas_from_nine_slice, NineSliceIndex, NineSliceSize,
    },
    ui::{
        action::{UiAction, UiActionPlugin},
        font_scale::{FontScale, FontSize},
        text_button,
    },
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;
use std::time::Duration;

/// Plugin providing functionality for the level select card/menu.
pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelSelect), spawn_level_select_card)
            .add_plugins((
                EventSchedulerPlugin::<LevelSelectCardEvent>::new(),
                UiActionPlugin::<LevelSelectAction>::new(),
            ))
            .add_systems(
                Update,
                (
                    pause.run_if(in_state(GameState::Graveyard)),
                    unpause.run_if(in_state(GameState::LevelSelect)),
                    select_level
                        .run_if(in_state(GameState::LevelSelect))
                        .run_if(on_event::<UiAction<LevelSelectAction>>()),
                    despawn_level_select_card.run_if(on_event::<LevelSelectCardEvent>()),
                ),
            )
            .add_systems(OnExit(GameState::LevelSelect), drop_level_select_card);
    }
}

/// Component that marks the level select UI card.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Component)]
pub struct LevelSelectCard;

/// Events regarding the visual state of the level select card.
#[derive(Event)]
pub enum LevelSelectCardEvent {
    /// Fires when the level select card entity is spawned.
    Spawned(Entity),
    /// Fires when the level select card entity has begun falling again.
    Falling(Entity),
    /// Fires when the level select card entity has fallen offscreen and should be despawned.
    Offscreen(Entity),
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum LevelSelectAction {
    GoToLevel(LevelSelection),
}

fn level_select_card_style(position: UiRect) -> Style {
    let UiRect {
        left,
        right,
        top,
        bottom,
    } = position;
    Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Absolute,
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        left,
        right,
        top,
        bottom,
        ..default()
    }
}

fn spawn_level_select_card(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    mut images: ResMut<Assets<Image>>,
    ldtk_assets: Res<Assets<LdtkProject>>,
    mut event_writer: EventWriter<LevelSelectCardEvent>,
) {
    // TODO: refactor this to avoid repeated code with spawn_level_card
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

    let level_select_entity = commands
        .spawn(ImageBundle {
            image: UiImage::new(level_card_texture),
            ..default()
        })
        .insert(
            level_select_card_style(UiRect {
                top: Val::Percent(100.),
                left: Val::Percent(0.),
                ..default()
            })
            .ease_to(
                level_select_card_style(UiRect {
                    top: Val::Percent(0.),
                    left: Val::Percent(0.),
                    ..default()
                }),
                EaseFunction::QuadraticOut,
                EasingType::Once {
                    duration: Duration::from_secs(1),
                },
            ),
        )
        .insert(LevelSelectCard)
        .with_children(|parent| {
            // spawn title
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "Level Select",
                        TextStyle {
                            font: asset_holder.font.clone(),
                            color: Color::WHITE,
                            ..default()
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    style: Style {
                        margin: UiRect {
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                            left: Val::Percent(10.),
                            right: Val::Percent(10.),
                        },
                        ..default()
                    },
                    ..default()
                })
                .insert(FontScale::from(FontSize::Huge));

            // spawn level button container
            parent
                .spawn(NodeBundle {
                    background_color: BackgroundColor(Color::NONE),
                    style: Style {
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::SpaceAround,
                        margin: UiRect {
                            top: Val::Px(10.),
                            bottom: Val::Px(10.),
                            left: Val::Percent(10.),
                            right: Val::Percent(10.),
                        },
                        width: Val::Percent(80.),
                        height: Val::Percent(60.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // spawn a button for every level
                    if let Some(ldtk) = ldtk_assets.get(&asset_holder.ldtk) {
                        for (i, _) in ldtk.iter_raw_levels().enumerate().skip(1) {
                            text_button::spawn(
                                parent,
                                format!("#{}", i),
                                &asset_holder,
                                Val::Percent(2.),
                                FontSize::Medium,
                            )
                            .insert(UiAction(
                                LevelSelectAction::GoToLevel(LevelSelection::index(i)),
                            ));
                        }
                    }
                });
        })
        .id();

    event_writer.send(LevelSelectCardEvent::Spawned(level_select_entity));
}

fn pause(mut next_state: ResMut<NextState<GameState>>, input: Res<ActionState<GraveyardAction>>) {
    if input.just_pressed(GraveyardAction::Pause) {
        next_state.set(GameState::LevelSelect);
    }
}

fn unpause(mut next_state: ResMut<NextState<GameState>>, input: Res<ActionState<GraveyardAction>>) {
    if input.just_pressed(GraveyardAction::Pause) {
        next_state.set(GameState::Graveyard);
    }
}

fn select_level(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut ui_actions: EventReader<UiAction<LevelSelectAction>>,
) {
    for action in ui_actions.read() {
        let UiAction(LevelSelectAction::GoToLevel(level_selection)) = action;
        commands.insert_resource(TransitionTo(level_selection.clone()));
        next_state.set(GameState::LevelTransition);
    }
}

fn drop_level_select_card(
    mut commands: Commands,
    level_select_card_query: Query<(Entity, &Style), With<LevelSelectCard>>,
    mut level_select_card_events: ResMut<EventScheduler<LevelSelectCardEvent>>,
) {
    for (entity, style) in level_select_card_query.iter() {
        commands.entity(entity).insert(
            level_select_card_style(UiRect {
                left: style.left,
                right: style.right,
                top: style.top,
                bottom: style.bottom,
            })
            .ease_to(
                level_select_card_style(UiRect {
                    top: Val::Percent(100.),
                    left: Val::Percent(0.),
                    ..default()
                }),
                EaseFunction::QuadraticOut,
                EasingType::Once {
                    duration: Duration::from_secs(1),
                },
            ),
        );

        // Demote level select card so it can't be doubly-despawned
        commands.entity(entity).remove::<LevelSelectCard>();

        level_select_card_events.schedule(
            LevelSelectCardEvent::Falling(entity),
            Duration::from_millis(0),
        );
        level_select_card_events.schedule(
            LevelSelectCardEvent::Offscreen(entity),
            Duration::from_millis(1000),
        );
    }
}

fn despawn_level_select_card(
    mut commands: Commands,
    mut level_select_card_events: EventReader<LevelSelectCardEvent>,
) {
    for event in level_select_card_events.read() {
        if let LevelSelectCardEvent::Offscreen(entity) = event {
            commands.entity(*entity).despawn_recursive();
        }
    }
}
