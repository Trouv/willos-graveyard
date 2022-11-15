use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct ExorcismBlock;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct DeathCard;

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
pub struct UiRoot;
