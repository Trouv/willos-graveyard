//! Plugin providing functionality for exorcism tiles, including death logic.
use crate::{
    graveyard::willo::WilloState,
    history::FlushHistoryCommands,
    ui::font_scale::{FontScale, FontSize},
    GameState,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use std::time::Duration;

/// Sets used by exorcism systems.
#[derive(SystemSet)]
pub enum ExorcismSets {
    /// Set used by the system that checks Willo's position kills Willo if appropriate.
    CheckDeath,
}

/// Plugin providing functionality for exorcism tiles, including death logic.
pub struct ExorcismPlugin;

impl Plugin for ExorcismPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExorcismEvent>()
            .add_system(
                check_death
                    .run_in_state(GameState::Graveyard)
                    .label(ExorcismSets::CheckDeath)
                    .after(FlushHistoryCommands),
            )
            .add_system_to_stage(CoreStage::PreUpdate, make_exorcism_card_visible)
            .add_system(spawn_death_card.run_in_state(GameState::Graveyard))
            .register_ldtk_int_cell::<ExorcismTileBundle>(2);
    }
}

/// Component that marks exorcism tiles.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ExorcismTile;

/// Event that fires when willo steps on an exorcism tile.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ExorcismEvent;

/// Component that marks the "Exorcized" card UI element.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ExorcismCard;

#[derive(Clone, Bundle, LdtkIntCell)]
struct ExorcismTileBundle {
    exorcism_tile: ExorcismTile,
}

fn check_death(
    mut willo_query: Query<(&GridCoords, &mut WilloState)>,
    exorcism_query: Query<(Entity, &GridCoords), With<ExorcismTile>>,
    mut death_event_writer: EventWriter<ExorcismEvent>,
) {
    if let Ok((coords, mut willo)) = willo_query.get_single_mut() {
        if *willo != WilloState::Dead && exorcism_query.iter().any(|(_, g)| *g == *coords) {
            *willo = WilloState::Dead;
            death_event_writer.send(ExorcismEvent);
        }
    }
}

fn spawn_death_card(
    mut commands: Commands,
    assets: Res<AssetServer>,
    willo_query: Query<&WilloState, Changed<WilloState>>,
    death_cards: Query<Entity, With<ExorcismCard>>,
    mut last_state: Local<WilloState>,
) {
    for state in willo_query.iter() {
        if *state == WilloState::Dead && *last_state != WilloState::Dead {
            // Player just died
            commands
                .spawn(NodeBundle {
                    background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.9)),
                    // The color renders before the transform is updated, so it needs to be
                    // invisible for the first update
                    visibility: Visibility { is_visible: false },
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
                            duration: Duration::from_millis(600),
                        },
                    ),
                )
                .insert(ExorcismCard)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle {
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
                            ..Default::default()
                        })
                        .insert(FontScale::from(FontSize::Medium));
                });
        } else if *state != WilloState::Dead && *last_state == WilloState::Dead {
            // Player just un-died
            if let Ok(entity) = death_cards.get_single() {
                commands.entity(entity).despawn_recursive();
            }
        }

        *last_state = *state;
    }
}

fn make_exorcism_card_visible(mut ui_query: Query<&mut Visibility, Added<ExorcismCard>>) {
    for mut visibility in ui_query.iter_mut() {
        visibility.is_visible = true;
    }
}
