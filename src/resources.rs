use bevy::prelude::*;
use std::{ops::Range, time::Duration};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct GoalGhostSettings {
    pub no_turn_length: Range<usize>,
    pub turn_length: Range<usize>,
    pub no_blink_length: Range<usize>,
    pub blink_length: Range<usize>,
    pub frame_duration: Duration,
    pub idle_frame_count: usize,
    pub happy_frame_count: usize,
    pub none_frame_index: usize,
    pub num_columns: usize,
    pub num_rows: usize,
    pub atlas: Option<Handle<TextureAtlas>>,
}

impl GoalGhostSettings {
    pub const NORMAL: GoalGhostSettings = GoalGhostSettings {
        no_turn_length: 32..64,
        turn_length: 12..20,
        no_blink_length: 50..100,
        blink_length: 0..1,
        frame_duration: Duration::from_millis(150),
        idle_frame_count: 8,
        happy_frame_count: 10,
        none_frame_index: 8,
        num_columns: 10,
        num_rows: 5,
        atlas: None,
    };
}

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

#[derive(Copy, Clone, PartialEq, Debug, Default, Deref, DerefMut)]
pub struct PlayZonePortion(pub f32);
