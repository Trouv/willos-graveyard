use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component, Serialize, Deserialize)]
pub enum RigidBody {
    Static,
    Dynamic,
}

#[derive(Copy, Clone, PartialEq, Debug, Default, Component, Serialize, Deserialize)]
pub struct Tile {
    pub coords: IVec2,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub struct InputBlock {
    pub key_code: KeyCode,
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
