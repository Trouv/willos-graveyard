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
pub struct DeathEvent {
    pub willo_entity: Entity,
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
