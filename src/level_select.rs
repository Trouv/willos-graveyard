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
        app.add_system(spawn_level_select_card.in_schedule(OnEnter(GameState::LevelSelect)))
            .add_plugin(EventSchedulerPlugin::<LevelSelectCardEvent>::new())
            .add_plugin(UiActionPlugin::<LevelSelectAction>::new())
            .add_system(pause.run_in_state(GameState::Graveyard))
            .add_system(unpause.run_in_state(GameState::LevelSelect))
            .add_system(
                select_level
                    .run_in_state(GameState::LevelSelect)
                    .run_on_event::<UiAction<LevelSelectAction>>(),
            )
            .add_system(drop_level_select_card.in_schedule(OnExit(GameState::LevelSelect)))
            .add_system(despawn_level_select_card.run_on_event::<LevelSelectCardEvent>());
    }
}

/// Component that marks the level select UI card.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Component)]
pub struct LevelSelectCard;

/// Events regarding the visual state of the level select card.
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
    Style {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Absolute,
        size: Size {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
        },
        position,
        ..default()
    }
}

fn spawn_level_select_card(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    mut images: ResMut<Assets<Image>>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
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
                        size: Size {
                            height: Val::Percent(60.),
                            width: Val::Percent(80.),
                        },
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // spawn a button for every level
                    if let Some(ldtk) = ldtk_assets.get(&asset_holder.ldtk) {
                        for (i, _) in ldtk.iter_levels().enumerate() {
                            text_button::spawn(
                                parent,
                                format!("#{}", i + 1),
                                &asset_holder,
                                Val::Percent(2.),
                                FontSize::Medium,
                            )
                            .insert(UiAction(
                                LevelSelectAction::GoToLevel(LevelSelection::Index(i)),
                            ));
                        }
                    }
                });
        })
        .id();

    event_writer.send(LevelSelectCardEvent::Spawned(level_select_entity));
}

fn pause(mut commands: Commands, input: Res<ActionState<GraveyardAction>>) {
    if input.just_pressed(GraveyardAction::Pause) {
        commands.insert_resource(NextState(GameState::LevelSelect));
    }
}

fn unpause(mut commands: Commands, input: Res<ActionState<GraveyardAction>>) {
    if input.just_pressed(GraveyardAction::Pause) {
        commands.insert_resource(NextState(GameState::Graveyard));
    }
}

fn select_level(mut commands: Commands, mut ui_actions: EventReader<UiAction<LevelSelectAction>>) {
    for action in ui_actions.iter() {
        let UiAction(LevelSelectAction::GoToLevel(level_selection)) = action;
        commands.insert_resource(NextState(GameState::Graveyard));
        commands.insert_resource(TransitionTo(level_selection.clone()));
        commands.insert_resource(NextState(GameState::LevelTransition));
    }
}

fn drop_level_select_card(
    mut commands: Commands,
    level_select_card_query: Query<(Entity, &Style), With<LevelSelectCard>>,
    mut level_select_card_events: ResMut<EventScheduler<LevelSelectCardEvent>>,
) {
    for (entity, style) in level_select_card_query.iter() {
        commands
            .entity(entity)
            .insert(level_select_card_style(style.position).ease_to(
                level_select_card_style(UiRect {
                    top: Val::Percent(100.),
                    left: Val::Percent(0.),
                    ..default()
                }),
                EaseFunction::QuadraticOut,
                EasingType::Once {
                    duration: Duration::from_secs(1),
                },
            ));

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
    for event in level_select_card_events.iter() {
        if let LevelSelectCardEvent::Offscreen(entity) = event {
            commands.entity(*entity).despawn_recursive();
        }
    }
}
