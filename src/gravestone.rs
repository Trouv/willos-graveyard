use crate::{history::History, sokoban::RigidBody, GameState};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};

pub struct GravestonePlugin;

impl Plugin for GravestonePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_gravestone_body.run_in_state(GameState::LevelTransition))
            .register_ldtk_entity::<InputBlockBundle>("W")
            .register_ldtk_entity::<InputBlockBundle>("A")
            .register_ldtk_entity::<InputBlockBundle>("S")
            .register_ldtk_entity::<InputBlockBundle>("D");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub struct InputBlock {
    pub key_code: KeyCode,
}

impl From<EntityInstance> for InputBlock {
    fn from(entity_instance: EntityInstance) -> Self {
        InputBlock {
            key_code: match entity_instance.identifier.as_ref() {
                "W" => KeyCode::W,
                "A" => KeyCode::A,
                "S" => KeyCode::S,
                _ => KeyCode::D,
            },
        }
    }
}

#[derive(Clone, Bundle, LdtkEntity)]
struct InputBlockBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    history: History<GridCoords>,
    #[from_entity_instance]
    rigid_body: RigidBody,
    #[from_entity_instance]
    input_block: InputBlock,
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

fn spawn_gravestone_body(
    mut commands: Commands,
    gravestones: Query<(Entity, &Handle<TextureAtlas>), Added<InputBlock>>,
) {
    for (entity, texture_handle) in gravestones.iter() {
        let index_range = 11..22_usize;

        let dist: Vec<usize> = (1..(index_range.len() + 1)).map(|x| x * x).rev().collect();

        let dist = WeightedIndex::new(dist).unwrap();

        let mut rng = rand::thread_rng();

        let body_entity = commands
            .spawn_bundle(SpriteSheetBundle {
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
