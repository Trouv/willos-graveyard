use crate::gameplay::Direction;
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct MoveTable {
    pub table: [[Option<KeyCode>; 4]; 4],
    pub player: Entity,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum PlayerState {
    Waiting,
    RankMove(KeyCode),
    FileMove(KeyCode),
}
