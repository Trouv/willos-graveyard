use bevy::prelude::*;
use std::{ops::Range, time::Duration};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct GoalGhostSettings {
    pub no_turn_length: Range<usize>,
    pub turn_length: Range<usize>,
    pub no_blink_length: Range<usize>,
    pub blink_length: Range<usize>,
    pub frame_duration: Duration,
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
        num_columns: 8,
        num_rows: 4,
        atlas: None,
    };
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct DeathAnimationTextureAtlases {
    pub death_hole_handle: Handle<TextureAtlas>,
    pub demon_arms_handle: Handle<TextureAtlas>,
}

pub fn load_death_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let death_hole_texture = asset_server.load("textures/animations/death-Sheet.png");
    let demon_arms_texture = asset_server.load("textures/animations/demon-Sheet.png");

    let death_hole_texture_atlas =
        TextureAtlas::from_grid(death_hole_texture, Vec2::splat(32.), 30, 1);
    let demon_arms_texture_atlas =
        TextureAtlas::from_grid(demon_arms_texture, Vec2::splat(32.), 30, 1);

    commands.insert_resource(DeathAnimationTextureAtlases {
        death_hole_handle: texture_atlases.add(death_hole_texture_atlas),
        demon_arms_handle: texture_atlases.add(demon_arms_texture_atlas),
    });
}
