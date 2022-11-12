use crate::{gameplay::components::*, *};
use bevy::prelude::*;
use std::cmp;

pub fn animate_grass_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut WindTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let mut rng = rand::thread_rng();
            let chance = rng.gen::<f32>();
            if chance <= 0.2 {
                sprite.index = cmp::min(sprite.index + 1, texture_atlas.len() - 1);
            } else if chance > 0.2 && chance <= 0.6 {
                sprite.index = cmp::max(sprite.index as i32 - 1, 0) as usize;
            }
        }
    }
}

pub fn make_ui_visible(mut ui_query: Query<&mut Visibility, With<Node>>) {
    for mut visibility in ui_query.iter_mut() {
        visibility.is_visible = true;
    }
}
