//! Plugin for providing the game's camera logic, fitting around the play zone and control-display.
use crate::{AssetHolder, GameState};
use bevy::{prelude::*, render::camera::ScalingMode, window::PrimaryWindow};
use bevy_ecs_ldtk::prelude::*;

/// Plugin for providing the game's camera logic, fitting around the play zone and control-display.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayZonePortion(0.75))
            .add_systems(Startup, spawn_camera)
            .add_systems(
                OnEnter(GameState::Graveyard),
                fit_camera_around_play_zone_padded,
            )
            .add_systems(
                Update,
                fit_camera_around_play_zone_padded
                    .run_if(not(in_state(GameState::AssetLoading)))
                    .run_if(on_event::<bevy::window::WindowResized>),
            );
    }
}

/// Resource for defining the percentage of the screen ([0-1]) that should be reserved for
/// rendering the level.
#[derive(Copy, Clone, PartialEq, Debug, Default, Deref, DerefMut, Resource)]
pub struct PlayZonePortion(pub f32);

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

fn fit_camera_around_play_zone_padded(
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    level_query: Query<&LevelIid>,
    project_assets: Res<Assets<LdtkProject>>,
    asset_holder: Res<AssetHolder>,
    windows: Query<&Window, With<PrimaryWindow>>,
    play_zone_portion: Res<PlayZonePortion>,
) {
    if let Ok(level_iid) = level_query.get_single() {
        let ldtk_project = project_assets
            .get(&asset_holder.ldtk)
            .expect("LDtk project should already be loaded");
        let level = ldtk_project
            .get_raw_level_by_iid(level_iid.get())
            .expect("level should exist in project");
        let level_size = IVec2::new(level.px_wid, level.px_hei);
        let padded_level_size = level_size + IVec2::splat(32 * 2);

        let window = windows.single();

        let padded_level_ratio = padded_level_size.x as f32 / padded_level_size.y as f32;
        let aspect_ratio = window.width() / window.height();
        let play_zone_ratio = aspect_ratio * **play_zone_portion;

        let (mut transform, mut projection) = camera_query.single_mut();
        projection.viewport_origin = Vec2::ZERO;

        let play_zone_size = if padded_level_ratio > play_zone_ratio {
            // Level is "wide"
            Vec2::new(
                padded_level_size.x as f32,
                padded_level_size.x as f32 / play_zone_ratio,
            )
        } else {
            // Level is "tall"
            Vec2::new(
                padded_level_size.y as f32 * play_zone_ratio,
                padded_level_size.y as f32,
            )
        };

        projection.scaling_mode = if play_zone_ratio > aspect_ratio {
            // Play zone is "wide"
            let pixel_perfect_width =
                ((play_zone_size.x / aspect_ratio).round() * aspect_ratio).round();

            ScalingMode::Fixed {
                width: pixel_perfect_width,
                height: (pixel_perfect_width / aspect_ratio).round(),
            }
        } else {
            // Play zone is "tall"

            let pixel_perfect_height =
                ((play_zone_size.y / aspect_ratio).round() * aspect_ratio).round();

            ScalingMode::Fixed {
                width: (pixel_perfect_height * aspect_ratio).round(),
                height: pixel_perfect_height,
            }
        };

        transform.translation.x = ((play_zone_size.x - padded_level_size.x as f32) / -2.).round();
        transform.translation.y = ((play_zone_size.y - padded_level_size.y as f32) / -2.).round();
    }
}
