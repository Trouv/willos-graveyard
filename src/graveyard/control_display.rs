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
                    justify_content: JustifyContent::Center,
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
    assets: Res<AssetServer>,
) {
    enum ControlNode {
        Text(String),
        Image(Handle<Image>),
    }

    for move_table in move_table_query.iter() {
        let control_display_entity = control_display_query.single();

        commands
            .entity(control_display_entity)
            .despawn_descendants();

        let font = assets.load("fonts/WayfarersToyBoxRegular-gxxER.ttf");

        let style = TextStyle {
            font,
            color: Color::WHITE,
            ..default()
        };
        commands
            .entity(control_display_entity)
            .with_children(|parent| {
                let mut add_row = |nodes: Vec<ControlNode>| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Percent(100. / 18.),
                                    ..Default::default()
                                },
                                margin: UiRect {
                                    bottom: Val::Px(16.),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            background_color: BackgroundColor(Color::NONE),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            for node in nodes {
                                match node {
                                    ControlNode::Text(s) => {
                                        parent
                                            .spawn(TextBundle {
                                                text: Text::from_section(s, style.clone())
                                                    .with_alignment(TextAlignment {
                                                        vertical: VerticalAlign::Center,
                                                        horizontal: HorizontalAlign::Center,
                                                    }),
                                                style: Style {
                                                    size: Size {
                                                        height: Val::Percent(100.),
                                                        ..Default::default()
                                                    },
                                                    margin: UiRect {
                                                        right: Val::Px(16.),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            })
                                            .insert(FontScale::from(FontSize::Medium));
                                    }
                                    ControlNode::Image(h) => {
                                        parent.spawn(ImageBundle {
                                            image: UiImage(h),
                                            style: Style {
                                                size: Size {
                                                    height: Val::Percent(100.),
                                                    ..Default::default()
                                                },
                                                margin: UiRect {
                                                    right: Val::Px(16.),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        });
                };

                let mut keys_to_controls: Vec<(GraveId, Vec<ControlNode>)> = vec![
                    (
                        GraveId::North,
                        vec![
                            ControlNode::Image(assets.load("textures/w.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                    (
                        GraveId::West,
                        vec![
                            ControlNode::Image(assets.load("textures/a.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                    (
                        GraveId::South,
                        vec![
                            ControlNode::Image(assets.load("textures/s.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                    (
                        GraveId::East,
                        vec![
                            ControlNode::Image(assets.load("textures/d.png")),
                            ControlNode::Text("=".to_string()),
                        ],
                    ),
                ];

                for (i, rank) in move_table.table.iter().enumerate() {
                    for (j, key) in rank.iter().enumerate() {
                        if let Some(key) = key {
                            let first_dir = DIRECTION_ORDER[i];
                            let second_dir = DIRECTION_ORDER[j];

                            let direction_handle = |d: Direction| -> Handle<Image> {
                                match d {
                                    Direction::Up => assets.load("textures/up.png"),
                                    Direction::Left => assets.load("textures/left.png"),
                                    Direction::Down => assets.load("textures/down.png"),
                                    Direction::Right => assets.load("textures/right.png"),
                                }
                            };

                            if let Some((_, controls)) =
                                keys_to_controls.iter_mut().find(|(k, _)| k == key)
                            {
                                controls.extend(vec![
                                    ControlNode::Image(direction_handle(first_dir)),
                                    ControlNode::Image(direction_handle(second_dir)),
                                ]);
                            }
                        }
                    }
                }

                keys_to_controls
                    .into_iter()
                    .for_each(|(_, row)| add_row(row));

                add_row(vec![
                    ControlNode::Text("R".to_string()),
                    ControlNode::Text("=".to_string()),
                    ControlNode::Text("restart".to_string()),
                ]);
                add_row(vec![
                    ControlNode::Text("Z".to_string()),
                    ControlNode::Text("=".to_string()),
                    ControlNode::Text("undo".to_string()),
                ]);
            });
    }
}
