//! Plugin providing the wind systems and components that react to it (currently just grass).
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use rand::Rng;
use std::cmp;

/// Plugin providing the wind systems and components that react to it (currently just grass).
pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_grass_system.run_if(not(in_state(GameState::AssetLoading))))
            .register_ldtk_entity::<GrassBundle>("Grass");
    }
}

/// Component with timer defining spacing between animation updates of wind-reactive components.
#[derive(Clone, Debug, Component)]
struct WindTimer(Timer);

impl Default for WindTimer {
    fn default() -> WindTimer {
        WindTimer(Timer::from_seconds(0.2, TimerMode::Repeating))
    }
}

#[derive(Clone, Bundle, LdtkEntity)]
struct GrassBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    wind_timer: WindTimer,
}

fn animate_grass_system(
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
