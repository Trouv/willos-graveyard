use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct MoveTable {
    pub table: [[Option<KeyCode>; 4]; 4],
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

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct History {
    pub tiles: Vec<GridCoords>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component, Serialize, Deserialize)]
pub enum LevelCard {
    Rising,
    Holding,
    Falling,
    End,
}

#[derive(Clone, Debug, Component)]
pub struct MovementTimer(pub Timer);

impl Default for MovementTimer {
    fn default() -> MovementTimer {
        MovementTimer(Timer::from_seconds(0.14, false))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct OrthographicCamera;
