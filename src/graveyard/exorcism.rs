//! Plugin providing functionality for exorcism tiles, including death logic.
use crate::{
    graveyard::{
        volatile::{Sublimation, Volatile},
        willo::WilloState,
    },
    history::{FlushHistoryCommands, History},
    ui::font_scale::{FontScale, FontSize},
    utils::any_match_filter,
    GameState,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use std::time::Duration;

/// Sets used by exorcism systems.
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum ExorcismSets {
    /// Set used by the system that checks Willo's position kills Willo if appropriate.
    CheckDeath,
}

/// Plugin providing functionality for exorcism tiles, including death logic.
pub struct ExorcismPlugin;

impl Plugin for ExorcismPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExorcismEvent>()
            .add_systems(
                Update,
                (
                    check_death
                        .run_if(in_state(GameState::Graveyard))
                        .in_set(ExorcismSets::CheckDeath)
                        .after(Sublimation),
                    spawn_death_card.run_if(in_state(GameState::Graveyard)),
                ),
            )
            .add_systems(PreUpdate, make_exorcism_card_visible)
            .register_ldtk_int_cell::<ExorcismTileBundle>(2);
    }
}

/// Event that fires when willo steps on an exorcism tile.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Event)]
pub struct ExorcismEvent;

/// Component that marks the "Exorcized" card UI element.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ExorcismCard;

#[derive(Clone, Bundle, LdtkIntCell)]
struct ExorcismTileBundle {
    volatile: Volatile,
    volatile_history: History<Volatile>,
}

fn check_death(
    mut willo_query: Query<(&mut WilloState, &Volatile), Changed<Volatile>>,
    mut death_event_writer: EventWriter<ExorcismEvent>,
) {
    if let Ok((mut willo, volatile)) = willo_query.get_single_mut() {
        if !volatile.is_solid() && *willo != WilloState::Dead {
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
                    visibility: Visibility::Hidden,
                    ..Default::default()
                })
                .insert(
                    Style {
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
                        Style {
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
                            .with_alignment(TextAlignment::Center),
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
        *visibility = Visibility::Inherited;
    }
}
