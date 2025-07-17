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
        app.add_systems(
            Update,
            animate_grass_system.run_if(not(in_state(GameState::AssetLoading))),
        )
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

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct GrassBundle {
    #[sprite_sheet]
    sprite_sheet: Sprite,
    wind_timer: WindTimer,
}

fn animate_grass_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut WindTimer, &mut Sprite)>,
) {
    for (
        mut timer,
        Sprite {
            texture_atlas: Some(mut texture_atlas),
            ..
        },
    ) in query.iter_mut()
    {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let texture_atlas_layout = texture_atlases.get(texture_atlas.layout).unwrap();
            let mut rng = rand::thread_rng();
            let chance = rng.gen::<f32>();
            if chance <= 0.2 {
                texture_atlas.index =
                    cmp::min(texture_atlas.index + 1, texture_atlas_layout.len() - 1);
            } else if chance > 0.2 && chance <= 0.6 {
                texture_atlas.index = cmp::max(texture_atlas.index as i32 - 1, 0) as usize;
            }
        }
    }
}
