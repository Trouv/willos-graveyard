//! Plugin providing functionality for the graveyard UI element showing the current controls.
use crate::{
    camera::PlayZonePortion,
    graveyard::{
        gravestone::GraveId,
        movement_table::{MovementTable, DIRECTION_ORDER},
    },
    sokoban::Direction,
    ui::font_scale::{FontScale, FontSize},
    GameState,
};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

/// Plugin providing functionality for the graveyard UI element showing the current controls.
pub struct ControlDisplayPlugin;

impl Plugin for ControlDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::LevelTransition, spawn_control_display)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_control_display.run_in_state(GameState::Graveyard),
            );
    }
}

/// Component that marks the main ControlDisplay UI Node.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ControlDisplay;

fn spawn_control_display(
    mut commands: Commands,
    play_zone_portion: Res<PlayZonePortion>,
    mut already_spawned: Local<bool>,
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
            .insert(ControlDisplay);

        *already_spawned = true;
    }
}

fn update_control_display(
    mut commands: Commands,
    move_table_query: Query<&MovementTable, Changed<MovementTable>>,
    control_display_query: Query<Entity, With<ControlDisplay>>,
) {
    for move_table in move_table_query.iter() {
        let control_display_entity = control_display_query.single();

        println!("spawning..");

        commands
            .entity(control_display_entity)
            .despawn_descendants();

        commands
            .entity(control_display_entity)
            .with_children(|control_display| {
                // spawn grave ids
                control_display
                    .spawn(NodeBundle {
                        style: Style {
                            aspect_ratio: Some(0.66666),
                            size: Size {
                                width: Val::Percent(80.),
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|grave_id_container| {});

                // spawn other grave actions
                control_display
                    .spawn(NodeBundle {
                        style: Style {
                            aspect_ratio: Some(0.33333),
                            size: Size {
                                width: Val::Percent(80.),
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|graveyard_action_container| {});
            });
    }
}
