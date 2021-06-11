use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Tile {
    pub coords: IVec2,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Blocker;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Pushable;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct InputBlock {
    pub key_code: KeyCode,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Goal;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub struct MoveTable {
    pub table: [[bool; 4]; 4],
    pub player: Entity,
}
