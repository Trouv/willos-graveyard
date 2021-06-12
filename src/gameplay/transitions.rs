use bevy::prelude::*;

use crate::{LevelSize, UNIT_LENGTH};

pub fn create_camera(mut commands: Commands, level_size: Res<LevelSize>) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    let scale = level_size.size.max_element() as f32;
    camera_bundle.transform.translation =
        Vec3::new((16.0 * UNIT_LENGTH) / 2.0, (9.0 * UNIT_LENGTH) / 2.0, 0.0);
    camera_bundle.orthographic_projection.far = 1000.0 / scale;
    camera_bundle.orthographic_projection.scale = scale;
    commands.spawn().insert_bundle(camera_bundle);
}
