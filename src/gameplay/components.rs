use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::*, utils::ldtk_grid_coords_to_tile_pos};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component, Serialize, Deserialize)]
pub enum RigidBody {
    Static,
    Dynamic,
}

impl From<EntityInstance> for RigidBody {
    fn from(_: EntityInstance) -> RigidBody {
        RigidBody::Dynamic
    }
}

impl From<IntGridCell> for RigidBody {
    fn from(_: IntGridCell) -> RigidBody {
        RigidBody::Static
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default, Component, Serialize, Deserialize)]
pub struct Tile {
    pub coords: IVec2,
}

impl LdtkEntity for Tile {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlas>,
    ) -> Self {
        let tile_pos = ldtk_grid_coords_to_tile_pos(entity_instance.grid, layer_instance.c_hei);
        Tile {
            coords: IVec2::new(tile_pos.0 as i32, tile_pos.1 as i32),
        }
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component, Serialize, Deserialize)]
pub struct Goal;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub struct MoveTable {
    pub table: [[Option<KeyCode>; 4]; 4],
    pub player: Entity,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum PlayerState {
    Waiting,
    RankMove(KeyCode),
    FileMove(KeyCode),
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState::Waiting
    }
}

#[derive(Clone, PartialEq, Debug, Default, Component, Serialize, Deserialize)]
pub struct History {
    pub tiles: Vec<Tile>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component, Serialize, Deserialize)]
pub enum LevelCard {
    Rising,
    Holding,
    Falling,
    End,
}
