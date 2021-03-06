use crate::UNIT_LENGTH;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
pub mod components;
pub mod systems;
pub mod transitions;

pub fn xy_translation(coords: IVec2) -> Vec2 {
    Vec2::new(coords.x as f32 + 0.5, coords.y as f32 + 0.5) * UNIT_LENGTH
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

pub const DIRECTION_ORDER: [Direction; 4] = [
    Direction::Up,
    Direction::Left,
    Direction::Down,
    Direction::Right,
];

impl From<Direction> for IVec2 {
    fn from(direction: Direction) -> IVec2 {
        match direction {
            Direction::Up => IVec2::Y,
            Direction::Left => IVec2::new(-1, 0),
            Direction::Down => IVec2::new(0, -1),
            Direction::Right => IVec2::X,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct PlayerMovementEvent {
    direction: Direction,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct DeathEvent {
    pub player_entity: Entity,
    pub exorcism_entity: Entity,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum LevelCardEvent {
    Rise(LevelSelection),
    Block(LevelSelection),
    Fall,
    Despawn,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GoalEvent {
    Met {
        goal_entity: Entity,
        stone_entity: Entity,
    },
    UnMet {
        goal_entity: Entity,
    },
}
