use crate::{
    gameplay::{components::*, xy_translation, Direction, MovementEvent},
    LevelSize,
};
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

fn push_tile_recursively(
    collision_map: Vec<Vec<Option<(Entity, RigidBody)>>>,
    pusher_coords: IVec2,
    direction: Direction,
) -> Vec<Entity> {
    let pusher = collision_map[pusher_coords.y as usize][pusher_coords.x as usize]
        .expect("pusher should exist")
        .0;
    let destination = pusher_coords + direction.into();
    if destination.x < 0
        || destination.y < 0
        || destination.y as usize >= collision_map.len()
        || destination.y as usize >= collision_map[0].len()
    {
        return Vec::new();
    }
    match collision_map[destination.y as usize][destination.x as usize] {
        None => vec![pusher],
        Some((_, RigidBody::Static)) => Vec::new(),
        Some((_, RigidBody::Dynamic)) => {
            let mut pushed_entities = push_tile_recursively(collision_map, destination, direction);
            if pushed_entities.is_empty() {
                Vec::new()
            } else {
                pushed_entities.push(pusher);
                pushed_entities
            }
        }
    }
}

pub fn perform_tile_movement(
    mut tile_query: Query<(Entity, &mut Tile, &RigidBody)>,
    mut reader: EventReader<MovementEvent>,
    level_size: Res<LevelSize>,
) {
    for movement_event in reader.iter() {
        let mut collision_map: Vec<Vec<Option<(Entity, RigidBody)>>> =
            vec![vec![None; level_size.size.x as usize]; level_size.size.y as usize];
        for (entity, tile, rigid_body) in tile_query.iter_mut() {
            collision_map[tile.coords.y as usize][tile.coords.x as usize] =
                Some((entity, *rigid_body));
        }

        let player_tile = tile_query
            .get_component::<Tile>(movement_event.player)
            .unwrap();

        let pushed_entities =
            push_tile_recursively(collision_map, player_tile.coords, movement_event.direction);

        for entity in pushed_entities {
            tile_query
                .get_component_mut::<Tile>(entity)
                .expect("Pushed should have Tile component")
                .coords += movement_event.direction.into();
        }
    }
}
