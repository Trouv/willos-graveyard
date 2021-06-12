use crate::gameplay::{components::*, xy_translation, Direction, MovementEvent};
use bevy::prelude::*;
use bevy_easings::*;

pub fn ease_movement(
    mut commands: Commands,
    mut tile_query: Query<(Entity, &Tile, &Transform), Changed<Tile>>,
) {
    for (entity, tile, transform) in tile_query.iter_mut() {
        let xy = xy_translation(tile.coords);

        commands.entity(entity).insert(transform.ease_to(
            Transform::from_xyz(xy.x, xy.y, transform.translation.z),
            EaseFunction::CubicOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(200),
            },
        ));
    }
}

pub fn simple_movement(
    player_query: Query<Entity, With<PlayerState>>,
    input: Res<Input<KeyCode>>,
    mut writer: EventWriter<MovementEvent>,
) {
    for player in player_query.iter() {
        if input.just_pressed(KeyCode::W) {
            writer.send(MovementEvent {
                player,
                direction: Direction::Up,
            });
        } else if input.just_pressed(KeyCode::A) {
            writer.send(MovementEvent {
                player,
                direction: Direction::Left,
            });
        } else if input.just_pressed(KeyCode::S) {
            writer.send(MovementEvent {
                player,
                direction: Direction::Down,
            });
        } else if input.just_pressed(KeyCode::D) {
            writer.send(MovementEvent {
                player,
                direction: Direction::Right,
            });
        }
    }
}

pub fn perform_tile_movement(
    mut tile_query: Query<&mut Tile, With<Pushable>>,
    mut reader: EventReader<MovementEvent>,
) {
    for movement_event in reader.iter() {
        let mut player_tile = tile_query.get_mut(movement_event.player).unwrap();
        player_tile.coords += movement_event.direction.into();
    }
}
