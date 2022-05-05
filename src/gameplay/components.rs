use crate::gameplay::Direction;
use bevy::{prelude::*, utils::Duration};
use bevy_ecs_ldtk::prelude::*;
use std::ops::Range;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct Goal;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct ExorcismBlock;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct MoveTable {
    pub table: [[Option<KeyCode>; 4]; 4],
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum PlayerState {
    Waiting,
    Dead,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum LevelCard {
    Rising,
    Holding,
    Falling,
    End,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct DeathCard;

const MOVEMENT_SECONDS: f32 = 0.14;

#[derive(Clone, Debug, Component)]
pub struct MovementTimer(pub Timer);

impl Default for MovementTimer {
    fn default() -> MovementTimer {
        MovementTimer(Timer::from_seconds(MOVEMENT_SECONDS, false))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct OrthographicCamera;

#[derive(Clone, Debug, Component)]
pub struct WindTimer(pub Timer);

impl Default for WindTimer {
    fn default() -> WindTimer {
        WindTimer(Timer::from_seconds(0.2, true))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct ControlDisplayNode;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct UiRoot;

#[derive(Clone, Debug, Default, Component)]
pub struct SpriteSheetAnimation {
    pub indices: Range<usize>,
    pub frame_timer: Timer,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Component)]
pub enum PlayerAnimationState {
    Idle,
    Push(Direction),
    Move(Direction),
    Dying,
    None,
}

impl Default for PlayerAnimationState {
    fn default() -> Self {
        PlayerAnimationState::Idle
    }
}

impl Iterator for PlayerAnimationState {
    type Item = Self;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self {
            PlayerAnimationState::Dying | PlayerAnimationState::None => PlayerAnimationState::None,
            PlayerAnimationState::Push(d) => PlayerAnimationState::Move(*d),
            _ => PlayerAnimationState::Idle,
        })
    }
}

impl From<PlayerAnimationState> for SpriteSheetAnimation {
    fn from(state: PlayerAnimationState) -> SpriteSheetAnimation {
        use Direction::*;
        use PlayerAnimationState::*;

        let indices = match state {
            Idle => 0..6,
            Move(Up) => 25..31,
            Move(Down) => 50..56,
            Move(Left) => 75..81,
            Move(Right) => 100..106,
            Push(Up) => 126..128,
            Push(Down) => 151..153,
            Push(Left) => 176..178,
            Push(Right) => 201..203,
            Dying => 225..250,
            None => 7..8,
        };

        let frame_timer = match state {
            _ => Timer::new(Duration::from_millis(150), true),
        };

        SpriteSheetAnimation {
            indices,
            frame_timer,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HandDirection {
    Right,
    Left,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GoalAnimationState {
    Idle,
    Turn { hand: HandDirection, frames: usize },
    Blinking { frames: usize },
}

impl Default for GoalAnimationState {
    fn default() -> Self {
        GoalAnimationState::Idle
    }
}

#[derive(Clone, Default, Debug, Component)]
pub struct GoalGhostAnimation {
    pub column: usize,
    pub frame_timer: Timer,
    pub frames_since_blink: usize,
    pub frames_since_turn: usize,
    pub state: GoalAnimationState,
}
