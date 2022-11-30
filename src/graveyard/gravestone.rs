//! Plugin and components providing functionality for gravestones.
//!
//! Gravestones are sokoban blocks that
//! - interact with goals to complete levels
//! - interact with the movement table to alter Willo's abilities
use crate::{
    graveyard::{
        sokoban::RigidBody,
        willo::{WilloLabels, WilloState},
    },
    history::{FlushHistoryCommands, History, HistoryCommands},
    GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

/// Plugin providing functionality for gravestones.
///
/// Gravestones are sokoban blocks that
/// - interact with goals to complete levels
/// - interact with the movement table to alter Willo's abilities
pub struct GravestonePlugin;

impl Plugin for GravestonePlugin {
    fn build(&self, app: &mut App) {
        let asset_folder = app.get_added_plugins::<AssetPlugin>()[0]
            .asset_folder
            .clone();

        app.add_plugin(InputManagerPlugin::<GraveId>::default())
            .init_resource::<ActionState<GraveId>>()
            .insert_resource(
                load_gravestone_control_settings(asset_folder)
                    .expect("unable to load gravestone control settings"),
            )
            .add_system(spawn_gravestone_body.run_in_state(GameState::LevelTransition))
            .add_system(
                gravestone_input
                    .run_in_state(GameState::Graveyard)
                    .label(WilloLabels::Input)
                    .before(FlushHistoryCommands),
            )
            .register_ldtk_entity::<GravestoneBundle>("W")
            .register_ldtk_entity::<GravestoneBundle>("A")
            .register_ldtk_entity::<GravestoneBundle>("S")
            .register_ldtk_entity::<GravestoneBundle>("D");
    }
}

/// Component that marks gravestones and associates them with an action.
///
/// Also acts as the grave-action itself by implementing Actionlike.
#[derive(
    Actionlike, Copy, Clone, PartialEq, Eq, Debug, Hash, Component, Serialize, Deserialize,
)]
pub enum GraveId {
    North,
    West,
    South,
    East,
}

fn load_gravestone_control_settings(asset_folder: String) -> std::io::Result<InputMap<GraveId>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(
        format!("{}/../settings/gravestone_controls.json", asset_folder),
    )?))?)
}

impl From<EntityInstance> for GraveId {
    fn from(entity_instance: EntityInstance) -> Self {
        match entity_instance.identifier.as_ref() {
            "W" => GraveId::North,
            "A" => GraveId::West,
            "S" => GraveId::South,
            "D" => GraveId::East,
            g => panic!("encountered bad gravestone identifier: {}", g),
        }
    }
}

#[derive(Clone, Bundle, LdtkEntity)]
struct GravestoneBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    history: History<GridCoords>,
    #[from_entity_instance]
    rigid_body: RigidBody,
    #[from_entity_instance]
    gravestone: GraveId,
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

fn spawn_gravestone_body(
    mut commands: Commands,
    gravestones: Query<(Entity, &Handle<TextureAtlas>), Added<GraveId>>,
) {
    for (entity, texture_handle) in gravestones.iter() {
        let index_range = 11..22_usize;

        let dist: Vec<usize> = (1..(index_range.len() + 1)).map(|x| x * x).rev().collect();

        let dist = WeightedIndex::new(dist).unwrap();

        let mut rng = rand::thread_rng();

        let body_entity = commands
            .spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: (11..22_usize).collect::<Vec<usize>>()[dist.sample(&mut rng)],
                    ..default()
                },
                texture_atlas: texture_handle.clone(),
                transform: Transform::from_xyz(0., 0., -0.5),
                ..default()
            })
            .id();

        commands.entity(entity).add_child(body_entity);
    }
}

fn gravestone_input(
    mut willo_query: Query<&mut WilloState>,
    grave_input: Res<ActionState<GraveId>>,
    mut history_commands: EventWriter<HistoryCommands>,
) {
    for mut willo in willo_query.iter_mut() {
        if *willo == WilloState::Waiting {
            if grave_input.just_pressed(GraveId::North) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(GraveId::North)
            } else if grave_input.just_pressed(GraveId::West) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(GraveId::West)
            } else if grave_input.just_pressed(GraveId::South) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(GraveId::South)
            } else if grave_input.just_pressed(GraveId::East) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(GraveId::East)
            }
        }
    }
}
