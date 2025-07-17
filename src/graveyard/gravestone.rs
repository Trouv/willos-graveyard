//! Plugin and components providing functionality for gravestones.
//!
//! Gravestones are sokoban blocks that
//! - interact with goals to complete levels
//! - interact with the movement table to alter Willo's abilities
use crate::{
    graveyard::{
        volatile::{Sublimation, Volatile},
        willo::{WilloSets, WilloState},
    },
    history::{FlushHistoryCommands, History, HistoryCommands},
    sokoban::SokobanBlock,
    ui::{action::UiActionPlugin, button_prompt::ButtonPromptPlugin},
    GameState,
};
use bevy::{prelude::*, reflect::Enum};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, ops::Range};

/// Plugin providing functionality for gravestones.
///
/// Gravestones are sokoban blocks that
/// - interact with goals to complete levels
/// - interact with the movement table to alter Willo's abilities
pub struct GravestonePlugin;

impl Plugin for GravestonePlugin {
    fn build(&self, app: &mut App) {
        let asset_folder = app.get_added_plugins::<AssetPlugin>()[0].file_path.clone();

        app.add_plugins((
            InputManagerPlugin::<GraveId>::default(),
            UiActionPlugin::<GraveId>::new(),
            ButtonPromptPlugin::<GraveId>::new(),
        ))
        .init_resource::<ActionState<GraveId>>()
        .init_resource::<GravestoneSettings>()
        .insert_resource(
            load_gravestone_control_settings(asset_folder)
                .expect("unable to load gravestone control settings"),
        )
        .add_systems(
            Update,
            (
                spawn_gravestone_body.run_if(in_state(GameState::LevelTransition)),
                sublimate_gravestones
                    .run_if(in_state(GameState::Graveyard))
                    .after(Sublimation),
                gravestone_input
                    .run_if(in_state(GameState::Graveyard))
                    .in_set(WilloSets::Input)
                    .before(FlushHistoryCommands),
            ),
        )
        .register_ldtk_entity::<GravestoneBundle>("Q")
        .register_ldtk_entity::<GravestoneBundle>("W")
        .register_ldtk_entity::<GravestoneBundle>("E")
        .register_ldtk_entity::<GravestoneBundle>("A")
        .register_ldtk_entity::<GravestoneBundle>("S")
        .register_ldtk_entity::<GravestoneBundle>("D");
    }
}

/// Asset collection for loading assets relevant to gravestones and gravestone controls.
#[derive(Debug, Default, AssetCollection, Resource)]
pub struct GravestoneAssets {
    #[asset(texture_atlas(tile_size_x = 32, tile_size_y = 32, columns = 10, rows = 2))]
    grave_bodies_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/graves-Sheet.png")]
    grave_bodies: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 16, tile_size_y = 16, columns = 16, rows = 11))]
    key_code_icons_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/key-code-icons.png")]
    key_code_icons: Handle<TextureAtlasLayout>,
}

#[derive(Debug, Resource)]
struct GravestoneSettings {
    gravestone_indices: Range<usize>,
    gravestone_translation: Vec3,
    icon_translation: Vec3,
}

impl Default for GravestoneSettings {
    fn default() -> Self {
        GravestoneSettings {
            gravestone_indices: 0..11,
            gravestone_translation: Vec3::ZERO,
            icon_translation: Vec3::new(0., 5., 0.1),
        }
    }
}

/// Component that marks gravestones and associates them with an action.
///
/// Also acts as the grave-action itself by implementing Actionlike.
#[derive(
    Actionlike,
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    Debug,
    Hash,
    Component,
    Serialize,
    Deserialize,
    Reflect,
)]
pub enum GraveId {
    /// Gravestone/action that applies to "northwesty" buttons like Q and Left Bumper.
    Northwest,
    #[default]
    /// Gravestone/action that applies to "northy" buttons like W and Triangle.
    North,
    /// Gravestone/action that applies to "northeasty" buttons like E and Right Bumper.
    Northeast,
    /// Gravestone/action that applies to "westy" buttons like A and Square.
    West,
    /// Gravestone/action that applies to "southy" buttons like S and X/Cross.
    South,
    /// Gravestone/action that applies to "northy" buttons like D and Circle.
    East,
}

fn load_gravestone_control_settings(asset_folder: String) -> std::io::Result<InputMap<GraveId>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(serde_json::from_reader(BufReader::new(File::open(
            format!("{asset_folder}/../settings/gravestone_controls.json"),
        )?))?)
    }

    // placed in a `#[cfg]` block rather than `if cfg!` so that changes to the file don't
    // recompile non-wasm builds.
    #[cfg(target_arch = "wasm32")]
    {
        Ok(serde_json::from_str(include_str!(
            "../../settings/gravestone_controls.json"
        ))?)
    }
}

impl From<&EntityInstance> for GraveId {
    fn from(entity_instance: &EntityInstance) -> Self {
        match entity_instance.identifier.as_ref() {
            "Q" => GraveId::Northwest,
            "W" => GraveId::North,
            "E" => GraveId::Northeast,
            "A" => GraveId::West,
            "S" => GraveId::South,
            "D" => GraveId::East,
            g => panic!("encountered bad gravestone identifier: {g}"),
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct GravestoneBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    history: History<GridCoords>,
    #[with(SokobanBlock::new_dynamic)]
    sokoban_block: SokobanBlock,
    #[from_entity_instance]
    gravestone: GraveId,
    volatile: Volatile,
    volatile_history: History<Volatile>,
}

fn spawn_gravestone_body(
    mut commands: Commands,
    gravestones: Query<(Entity, &GraveId), Added<GraveId>>,
    assets: Res<GravestoneAssets>,
    settings: Res<GravestoneSettings>,
    input_map: Res<InputMap<GraveId>>,
) {
    for (entity, grave_id) in gravestones.iter() {
        commands.entity(entity).with_children(|parent| {
            let dist: Vec<usize> = (1..(settings.gravestone_indices.len() + 1))
                .map(|x| x * x)
                .rev()
                .collect();

            let dist = WeightedIndex::new(dist).unwrap();

            let mut rng = rand::thread_rng();

            // body entity
            parent.spawn((
                Sprite {
                    image: assets.grave_bodies.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: assets.grave_bodies_layout.clone(),
                        index: settings.gravestone_indices.clone().collect::<Vec<usize>>()
                            [dist.sample(&mut rng)],
                    }),
                    ..default()
                },
                Transform::from_translation(settings.gravestone_translation),
            ));

            // icon entity
            if let Some(key_code) = input_map
                .get_buttonlike(grave_id)
                .iter()
                .flat_map(|inputs| inputs.iter())
                .find_map(|i| i.as_any().downcast_ref::<KeyCode>())
            {
                parent.spawn((
                    Sprite {
                        image: assets.key_code_icons.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: assets.key_code_icons_layout.clone(),
                            index: key_code.variant_index(),
                        }),
                        ..default()
                    },
                    Transform::from_translation(settings.icon_translation),
                ));
            }
        });
    }
}

fn sublimate_gravestones(
    mut commands: Commands,
    mut gravestone_query: Query<
        (Entity, &Volatile, &mut Visibility),
        (With<GraveId>, Changed<Volatile>),
    >,
) {
    for (gravestone_entity, volatile, mut visibility) in gravestone_query.iter_mut() {
        match volatile {
            Volatile::Solid => {
                *visibility = Visibility::Inherited;
                commands
                    .entity(gravestone_entity)
                    .insert(SokobanBlock::Dynamic);
            }
            Volatile::Sublimated => {
                *visibility = Visibility::Hidden;
                commands.entity(gravestone_entity).remove::<SokobanBlock>();
            }
        }
    }
}

fn gravestone_input(
    mut willo_query: Query<&mut WilloState>,
    grave_input: Res<ActionState<GraveId>>,
    mut history_commands: EventWriter<HistoryCommands>,
) {
    for mut willo in willo_query.iter_mut() {
        if *willo == WilloState::Waiting {
            if grave_input.just_pressed(GraveId::Northwest) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(GraveId::Northwest)
            } else if grave_input.just_pressed(GraveId::North) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(GraveId::North)
            } else if grave_input.just_pressed(GraveId::Northeast) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(GraveId::Northeast)
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
