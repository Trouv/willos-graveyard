use crate::UNIT_LENGTH;
use bevy::prelude::*;

pub fn xy_translation(coords: IVec2) -> Vec2 {
    Vec2::new(coords.x as f32 + 0.5, coords.y as f32 + 0.5) * UNIT_LENGTH
}
