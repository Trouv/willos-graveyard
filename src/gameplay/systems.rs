use crate::gameplay::{components::*, xy_translation};
use bevy::prelude::*;
use bevy_easings::*;

pub fn ease_movement(mut tile_query: Query<(&Tile, &mut Transform), Changed<Tile>>) {
    for (tile, mut transform) in tile_query.iter_mut() {
        let xy = xy_translation(tile.coords);
        transform.ease_to(
            Transform::from_xyz(xy.x, xy.y, transform.translation.z),
            EaseFunction::SineOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(300),
            },
        );
    }
}
