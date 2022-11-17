use bevy::prelude::*;
use std::{ops::Range, time::Duration};

#[derive(Clone, Debug, Default)]
pub struct RewindTimer {
    pub velocity: f32,
    pub timer: Timer,
}

impl RewindTimer {
    pub fn new(millis: u64) -> RewindTimer {
        RewindTimer {
            velocity: millis as f32,
            timer: Timer::new(Duration::from_millis(millis), true),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RewindSettings {
    pub hold_range_millis: Range<u64>,
    pub hold_acceleration: f32,
    pub hold_timer: Option<RewindTimer>,
}

impl RewindSettings {
    pub const NORMAL: RewindSettings = RewindSettings {
        hold_range_millis: 50..200,
        hold_acceleration: 50.,
        hold_timer: None,
    };
}
