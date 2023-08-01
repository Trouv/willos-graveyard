//! Plugin providing functionality for the graveyard UI element showing the current controls.
use crate::{
    camera::PlayZonePortion,
    graveyard::{gravestone::GraveId, movement_table::MovementTable},
    ui::{
        action::UiAction,
        icon_button::{IconButton, IconButtonBundle, IconButtonSet},
    },
    ui_atlas_image::UiAtlasImage,
    GameState,
};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::GraveyardAction;

/// Plugin providing functionality for the graveyard UI element showing the current controls.
pub struct ControlDisplayPlugin;

impl Plugin for ControlDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::LevelTransition), spawn_control_display)
            .add_systems(
                Update,
                update_grave_action_buttons
                    .run_if(in_state(GameState::Graveyard))
                    .before(IconButtonSet),
            );
    }
}

/// Component that marks the main ControlDisplay UI Node.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct ControlDisplay;

/// Asset collection for loading/storing assets relevant to the control display.
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
                    width: Val::Percent(100. * control_zone_ratio),
                    height: Val::Percent(100.),
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
                            aspect_ratio: Some(3. / 2.),
                            width: Val::Percent(80.),
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
                            aspect_ratio: Some(3.),
                            width: Val::Percent(80.),
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

#[cfg(test)]
mod tests {
    use bevy::asset::HandleId;

    use super::*;

    fn app_setup() -> App {
        let mut app = App::new();

        app.add_state::<GameState>()
            .add_plugin(ControlDisplayPlugin)
            .insert_resource(PlayZonePortion(0.5));

        app
    }

    fn asset_setup(app: &mut App) -> ControlDisplayAssets {
        let control_display_assets = ControlDisplayAssets {
            movement_table_actions: Handle::weak(HandleId::random::<TextureAtlas>()),
            graveyard_actions: Handle::weak(HandleId::random::<TextureAtlas>()),
        };

        app.insert_resource(control_display_assets.clone());

        control_display_assets
    }

    fn spawn_movement_table(app: &mut App) -> Entity {
        app.world
            .spawn(MovementTable {
                table: [
                    [Some(GraveId::North), None, None, None],
                    [None, Some(GraveId::West), None, None],
                    [None, None, Some(GraveId::South), None],
                    [None, None, None, Some(GraveId::East)],
                ],
            })
            .id()
    }

    fn initial_state_changes(app: &mut App) {
        app.update();

        app.world
            .insert_resource(NextState(Some(GameState::LevelTransition)));

        app.update();

        app.world
            .insert_resource(NextState(Some(GameState::Graveyard)));

        app.update();
    }

    fn get_icon_button_for_action<A: Clone + Send + Sync + PartialEq + 'static>(
        app: &mut App,
        action: A,
    ) -> &IconButton {
        app.world
            .query::<(&IconButton, &UiAction<A>)>()
            .iter(&app.world)
            .find(|(_, a)| ***a == action)
            .map(|(i, _)| i)
            .unwrap()
    }

    #[test]
    fn plugin_spawns_all_buttons() {
        let mut app = app_setup();
        let assets = asset_setup(&mut app);
        spawn_movement_table(&mut app);
        initial_state_changes(&mut app);

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::North),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 0
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::West),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 5
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::South),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 10
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::East),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 15
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveyardAction::Undo),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.graveyard_actions.clone(),
                index: 0
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveyardAction::Restart),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.graveyard_actions.clone(),
                index: 1
            }),
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveyardAction::Pause),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.graveyard_actions.clone(),
                index: 2
            }),
        );
    }

    #[test]
    fn grave_id_buttons_change_according_to_movement_table() {
        let mut app = app_setup();
        let assets = asset_setup(&mut app);
        let movement_table_entity = spawn_movement_table(&mut app);
        initial_state_changes(&mut app);

        // check initial values of a couple buttons
        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::North),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 0
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::West),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 5
            }),
        );

        // change the movement table and check those buttons again
        let mut movement_table_mut = app.world.entity_mut(movement_table_entity);

        let mut movement_table = movement_table_mut.get_mut::<MovementTable>().unwrap();

        movement_table.table[0][0] = None;
        movement_table.table[0][1] = Some(GraveId::North);
        movement_table.table[1][1] = None;

        app.update();

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::North),
            &IconButton::AtlasImageIcon(UiAtlasImage {
                texture_atlas: assets.movement_table_actions.clone(),
                index: 1
            })
        );

        assert_eq!(
            get_icon_button_for_action(&mut app, GraveId::West),
            &IconButton::NoIcon,
        );
    }
}
