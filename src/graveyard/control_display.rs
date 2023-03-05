//! Plugin providing functionality for the graveyard UI element showing the current controls.
use crate::{
    camera::PlayZonePortion,
    graveyard::{
        gravestone::GraveId,
        movement_table::{MovementTable, DIRECTION_ORDER},
    },
    sokoban::Direction,
    ui::{
        action::UiAction,
        font_scale::{FontScale, FontSize},
        icon_button::{IconButton, IconButtonBundle, IconButtonLabel},
    },
    ui_atlas_image::UiAtlasImage,
    GameState,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::*;

use super::GraveyardAction;

/// Plugin providing functionality for the graveyard UI element showing the current controls.
pub struct ControlDisplayPlugin;

impl Plugin for ControlDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::LevelTransition, spawn_control_display)
            .add_system(
                update_grave_action_buttons
                    .run_in_state(GameState::Graveyard)
                    .before(IconButtonLabel),
            );
    }
}

/// Component that marks the main ControlDisplay UI Node.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ControlDisplay;

#[derive(Clone, Debug, AssetCollection, Resource)]
pub struct ControlDisplayAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 4, rows = 4))]
    #[asset(path = "textures/movement-table-actions.png")]
    movement_table_actions: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 3, rows = 1))]
    #[asset(path = "textures/graveyard-actions.png")]
    graveyard_actions: Handle<TextureAtlas>,
}

fn spawn_control_display(
    mut commands: Commands,
    play_zone_portion: Res<PlayZonePortion>,
    mut already_spawned: Local<bool>,
    assets: Res<ControlDisplayAssets>,
) {
    if !*already_spawned {
        let control_zone_ratio = 1. - **play_zone_portion;

        commands
            .spawn(NodeBundle {
                background_color: BackgroundColor(Color::NONE),
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::SpaceAround,
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
                z_index: ZIndex::Local(-1),
                ..Default::default()
            })
            .insert(ControlDisplay)
            .with_children(|control_display| {
                // spawn grave ids
                control_display
                    .spawn(NodeBundle {
                        style: Style {
                            aspect_ratio: Some((0.8 * 2.) / 3.),
                            size: Size {
                                width: Val::Percent(80.),
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|movement_table_action_container| {
                        // spawn north
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(0.),
                                    left: Val::Percent(100. / 3.),
                                    bottom: Val::Percent(50.),
                                    right: Val::Percent(100. / 3.),
                                },
                            ))
                            .insert(UiAction(GraveId::North));

                        // spawn west
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(50.),
                                    left: Val::Percent(0.),
                                    bottom: Val::Percent(0.),
                                    right: Val::Percent(200. / 3.),
                                },
                            ))
                            .insert(UiAction(GraveId::West));

                        // spawn south
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(50.),
                                    left: Val::Percent(100. / 3.),
                                    bottom: Val::Percent(0.),
                                    right: Val::Percent(100. / 3.),
                                },
                            ))
                            .insert(UiAction(GraveId::South));

                        // spawn east
                        movement_table_action_container
                            .spawn(IconButtonBundle::new_with_absolute_position(
                                IconButton::NoIcon,
                                UiRect {
                                    top: Val::Percent(50.),
                                    left: Val::Percent(200. / 3.),
                                    bottom: Val::Percent(0.),
                                    right: Val::Percent(0.),
                                },
                            ))
                            .insert(UiAction(GraveId::East));
                    });

                // spawn other grave actions
                control_display
                    .spawn(NodeBundle {
                        style: Style {
                            aspect_ratio: Some((1. * 0.8) / 3.),
                            size: Size {
                                width: Val::Percent(80.),
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|graveyard_action_container| {
                        // spawn undo
                        graveyard_action_container
                            .spawn(IconButtonBundle::new(IconButton::AtlasImageIcon(
                                UiAtlasImage {
                                    texture_atlas: assets.graveyard_actions.clone(),
                                    index: 0,
                                },
                            )))
                            .insert(UiAction(GraveyardAction::Undo));

                        // spawn restart
                        graveyard_action_container
                            .spawn(IconButtonBundle::new(IconButton::AtlasImageIcon(
                                UiAtlasImage {
                                    texture_atlas: assets.graveyard_actions.clone(),
                                    index: 1,
                                },
                            )))
                            .insert(UiAction(GraveyardAction::Restart));

                        // spawn pause
                        graveyard_action_container
                            .spawn(IconButtonBundle::new(IconButton::AtlasImageIcon(
                                UiAtlasImage {
                                    texture_atlas: assets.graveyard_actions.clone(),
                                    index: 2,
                                },
                            )))
                            .insert(UiAction(GraveyardAction::Pause));
                    });
            });

        *already_spawned = true;
    }
}

fn update_grave_action_buttons(
    movement_tables: Query<&MovementTable, Changed<MovementTable>>,
    mut grave_action_buttons: Query<(&mut IconButton, &UiAction<GraveId>)>,
    assets: Res<ControlDisplayAssets>,
) {
    for movement_table in movement_tables.iter() {
        for (mut icon_button, action) in &mut grave_action_buttons {
            *icon_button = match movement_table
                .table
                .iter()
                .flat_map(|row| row.iter())
                .enumerate()
                .find(|(_, g)| **g == Some(**action))
            {
                Some((index, _)) => IconButton::AtlasImageIcon(UiAtlasImage {
                    texture_atlas: assets.movement_table_actions.clone(),
                    index,
                }),
                None => IconButton::NoIcon,
            }
        }
    }
}
