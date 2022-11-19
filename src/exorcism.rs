//! Plugin providing functionality for exorcism tiles, including death logic.
use crate::{
    history::FlushHistoryCommands,
    ui::{
        font_scale::{FontScale, FontSize},
        UiRoot,
    },
    willo::WilloState,
    GameState, SystemLabels,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

/// Plugin providing functionality for exorcism tiles, including death logic.
pub struct ExorcismPlugin;

impl Plugin for ExorcismPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExorcismEvent>()
            .add_system(
                check_death
                    .run_in_state(GameState::Gameplay)
                    .label(SystemLabels::CheckDeath)
                    .after(FlushHistoryCommands),
            )
            .add_system_to_stage(CoreStage::PreUpdate, make_exorcism_card_visible)
            .add_system(spawn_death_card.run_in_state(GameState::Gameplay))
            .register_ldtk_int_cell::<ExorcismTileBundle>(2);
    }
}

/// Component that marks exorcism tiles.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ExorcismTile;

/// Event that fires when willo steps on an exorcism tile.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ExorcismEvent {
    pub willo_entity: Entity,
}

/// Component that marks the "Exorcized" card UI element.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ExorcismCard;

#[derive(Clone, Bundle, LdtkIntCell)]
struct ExorcismTileBundle {
    exorcism_tile: ExorcismTile,
}

fn check_death(
    mut willo_query: Query<(Entity, &GridCoords, &mut WilloState)>,
    exorcism_query: Query<(Entity, &GridCoords), With<ExorcismTile>>,
    mut death_event_writer: EventWriter<ExorcismEvent>,
) {
    if let Ok((entity, coords, mut willo)) = willo_query.get_single_mut() {
        if *willo != WilloState::Dead && exorcism_query.iter().any(|(_, g)| *g == *coords) {
            *willo = WilloState::Dead;
            death_event_writer.send(ExorcismEvent {
                willo_entity: entity,
            });
        }
    }
}

fn spawn_death_card(
    mut commands: Commands,
    assets: Res<AssetServer>,
    willo_query: Query<&WilloState, Changed<WilloState>>,
    death_cards: Query<Entity, With<ExorcismCard>>,
    mut last_state: Local<WilloState>,
    ui_root_query: Query<Entity, With<UiRoot>>,
) {
    for state in willo_query.iter() {
        if *state == WilloState::Dead && *last_state != WilloState::Dead {
            // Player just died
            let death_card_entity = commands
                .spawn_bundle(NodeBundle {
                    color: UiColor(Color::rgba(0., 0., 0., 0.9)),
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
                .insert(ExorcismCard)
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

fn make_exorcism_card_visible(mut ui_query: Query<&mut Visibility, Added<ExorcismCard>>) {
    for mut visibility in ui_query.iter_mut() {
        visibility.is_visible = true;
    }
}
