//! Plugin for providing the game's camera logic, fitting around the play zone and control-display.
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

/// Plugin for providing the game's camera logic, fitting around the play zone and control-display.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayZonePortion(0.75))
            .add_startup_system(spawn_camera)
            .add_enter_system(GameState::Gameplay, fit_camera_around_play_zone_padded)
            .add_system(
                fit_camera_around_play_zone_padded
                    .run_not_in_state(GameState::AssetLoading)
                    .run_on_event::<bevy::window::WindowResized>(),
            );
    }
}

/// Resource for defining the percentage of the screen ([0-1]) that should be reserved for
/// rendering the level.
#[derive(Copy, Clone, PartialEq, Debug, Default, Deref, DerefMut)]
pub struct PlayZonePortion(pub f32);

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn fit_camera_around_play_zone_padded(
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    level_query: Query<&Handle<LdtkLevel>>,
    levels: Res<Assets<LdtkLevel>>,
    windows: Res<Windows>,
    play_zone_portion: Res<PlayZonePortion>,
) {
    if let Ok(level_handle) = level_query.get_single() {
        if let Some(level) = levels.get(level_handle) {
            let level_size = IVec2::new(level.level.px_wid, level.level.px_hei);
            let padded_level_size = level_size + IVec2::splat(32 * 2);

            let window = windows.primary();

            let padded_level_ratio = padded_level_size.x as f32 / padded_level_size.y as f32;
            let aspect_ratio = window.width() / window.height();
            let play_zone_ratio = aspect_ratio * **play_zone_portion;

            let (mut transform, mut projection) = camera_query.single_mut();
            projection.scaling_mode = bevy::render::camera::ScalingMode::None;
            projection.bottom = 0.;
            projection.left = 0.;

            let play_zone_size = if padded_level_ratio > play_zone_ratio {
                // Level is "wide"
                Size {
                    width: padded_level_size.x as f32,
                    height: padded_level_size.x as f32 / play_zone_ratio,
                }
            } else {
                // Level is "tall"
                Size {
                    width: padded_level_size.y as f32 * play_zone_ratio,
                    height: padded_level_size.y as f32,
                }
            };

            if play_zone_ratio > aspect_ratio {
                // Play zone is "wide"
                let pixel_perfect_width =
                    ((play_zone_size.width / aspect_ratio).round() * aspect_ratio).round();

                projection.right = pixel_perfect_width;
                projection.top = (pixel_perfect_width / aspect_ratio).round();
            } else {
                // Play zone is "tall"

                let pixel_perfect_height =
                    ((play_zone_size.height / aspect_ratio).round() * aspect_ratio).round();

                projection.right = (pixel_perfect_height * aspect_ratio).round();
                projection.top = pixel_perfect_height;
            };

            transform.translation.x =
                ((play_zone_size.width - padded_level_size.x as f32) / -2.).round();
            transform.translation.y =
                ((play_zone_size.height - padded_level_size.y as f32) / -2.).round();
        }
    }
}
