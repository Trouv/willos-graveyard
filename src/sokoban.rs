use crate::gameplay::Direction;
use crate::{
    gameplay::{components::*, xy_translation},
    willo::{PlayerAnimationState, PlayerMovementEvent},
    *,
};
use bevy::prelude::*;
use bevy_easings::*;

pub fn ease_movement(
    mut commands: Commands,
    mut grid_coords_query: Query<
        (
            Entity,
            &GridCoords,
            &Transform,
            Option<&PlayerAnimationState>,
        ),
        (Changed<GridCoords>, Without<MoveTable>),
    >,
) {
    for (entity, &grid_coords, transform, player_state) in grid_coords_query.iter_mut() {
        let mut xy = xy_translation(grid_coords.into());

        if let Some(PlayerAnimationState::Push(direction)) = player_state {
            xy += IVec2::from(*direction).as_vec2() * 5.;
        }

        commands.entity(entity).insert(transform.ease_to(
            Transform::from_xyz(xy.x, xy.y, transform.translation.z),
            EaseFunction::CubicOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(110),
            },
        ));
    }
}

fn push_grid_coords_recursively(
    collision_map: Vec<Vec<Option<(Entity, RigidBody)>>>,
    pusher_coords: IVec2,
    direction: Direction,
) -> Vec<Entity> {
    let pusher = collision_map[pusher_coords.y as usize][pusher_coords.x as usize]
        .expect("pusher should exist")
        .0;
    let destination = pusher_coords + IVec2::from(direction);
    if destination.x < 0
        || destination.y < 0
        || destination.y as usize >= collision_map.len()
        || destination.x as usize >= collision_map[0].len()
    {
        return Vec::new();
    }
    match collision_map[destination.y as usize][destination.x as usize] {
        None => vec![pusher],
        Some((_, RigidBody::Static)) => Vec::new(),
        Some((_, RigidBody::Dynamic)) => {
            let mut pushed_entities =
                push_grid_coords_recursively(collision_map, destination, direction);
            if pushed_entities.is_empty() {
                Vec::new()
            } else {
                pushed_entities.push(pusher);
                pushed_entities
            }
        }
    }
}

pub fn perform_grid_coords_movement(
    mut grid_coords_query: Query<(
        Entity,
        &mut GridCoords,
        &RigidBody,
        Option<&mut PlayerAnimationState>,
    )>,
    mut reader: EventReader<PlayerMovementEvent>,
    level_query: Query<&Handle<LdtkLevel>>,
    levels: Res<Assets<LdtkLevel>>,
    audio: Res<Audio>,
    sfx: Res<AssetHolder>,
) {
    for movement_event in reader.iter() {
        let level = levels
            .get(level_query.single())
            .expect("Level should be loaded in gameplay state");

        let LayerInstance {
            c_wid: width,
            c_hei: height,
            ..
        } = level
            .level
            .layer_instances
            .clone()
            .expect("Loaded level should have layers")[0];

        let mut collision_map: Vec<Vec<Option<(Entity, RigidBody)>>> =
            vec![vec![None; width as usize]; height as usize];

        for (entity, grid_coords, rigid_body, _) in grid_coords_query.iter_mut() {
            collision_map[grid_coords.y as usize][grid_coords.x as usize] =
                Some((entity, *rigid_body));
        }

        if let Some((_, player_grid_coords, _, Some(mut animation))) = grid_coords_query
            .iter_mut()
            .find(|(_, _, _, animation)| animation.is_some())
        {
            let pushed_entities = push_grid_coords_recursively(
                collision_map,
                IVec2::from(*player_grid_coords),
                movement_event.direction,
            );

            if pushed_entities.len() > 1 {
                audio.play(sfx.push_sound.clone_weak());
                *animation.into_inner() = PlayerAnimationState::Push(movement_event.direction);
            } else {
                let new_state = PlayerAnimationState::Idle(movement_event.direction);
                if *animation != new_state {
                    *animation = new_state;
                }
            }

            for entity in pushed_entities {
                *grid_coords_query
                    .get_component_mut::<GridCoords>(entity)
                    .expect("Pushed should have GridCoords component") +=
                    GridCoords::from(IVec2::from(movement_event.direction));
            }
        }
    }
}

pub fn move_table_update(
    mut table_query: Query<(&GridCoords, &mut MoveTable)>,
    input_block_query: Query<(&GridCoords, &InputBlock)>,
) {
    for (table_grid_coords, mut table) in table_query.iter_mut() {
        table.table = [[None; 4]; 4];
        for (input_grid_coords, input_block) in input_block_query.iter() {
            let diff = *input_grid_coords - *table_grid_coords;
            let x_index = diff.x - 1;
            let y_index = -1 - diff.y;
            if (0..4).contains(&x_index) && (0..4).contains(&y_index) {
                // key block is in table
                table.table[y_index as usize][x_index as usize] = Some(input_block.key_code);
            }
        }
    }
}