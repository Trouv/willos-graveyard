//! Plugin and components providing functionality for sokoban-style movement and collision.
use crate::{
    from_component::FromComponentLabel,
    graveyard::{
        movement_table::{Direction, MovementTable},
        willo::{WilloAnimationState, WilloMovementEvent},
    },
    AssetHolder, GameState, UNIT_LENGTH,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use iyes_loopless::prelude::*;

/// Labels used by sokoban systems
#[derive(SystemLabel)]
pub enum SokobanLabels {
    EaseMovement,
    GridCoordsMovement,
}

/// Plugin providing functionality for sokoban-style movement and collision.
pub struct SokobanPlugin;

impl Plugin for SokobanPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            perform_grid_coords_movement
                .run_in_state(GameState::Graveyard)
                .run_on_event::<WilloMovementEvent>()
                .label(SokobanLabels::GridCoordsMovement)
                .before(FromComponentLabel),
        )
        // Systems with potential easing end/beginning collisions cannot be in CoreStage::Update
        // see https://github.com/vleue/bevy_easings/issues/23
        .add_system_to_stage(
            CoreStage::PostUpdate,
            ease_movement
                .run_not_in_state(GameState::AssetLoading)
                .label(SokobanLabels::EaseMovement),
        )
        .register_ldtk_int_cell::<WallBundle>(1)
        .register_ldtk_int_cell::<WallBundle>(3)
        .register_ldtk_int_cell::<WallBundle>(4);
    }
}

/// Component defining the behavior of sokoban entities on collision.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum RigidBody {
    Static,
    Dynamic,
}

#[derive(Clone, Bundle, LdtkIntCell)]
struct WallBundle {
    #[from_int_grid_cell]
    rigid_body: RigidBody,
}

impl From<EntityInstance> for RigidBody {
    fn from(_: EntityInstance) -> RigidBody {
        RigidBody::Dynamic
    }
}

impl From<IntGridCell> for RigidBody {
    fn from(_: IntGridCell) -> RigidBody {
        RigidBody::Static
    }
}

fn ease_movement(
    mut commands: Commands,
    mut grid_coords_query: Query<
        (
            Entity,
            &GridCoords,
            &Transform,
            Option<&WilloAnimationState>,
        ),
        (Changed<GridCoords>, Without<MovementTable>),
    >,
) {
    for (entity, &grid_coords, transform, willo_animation_state) in grid_coords_query.iter_mut() {
        let mut xy = grid_coords_to_translation(grid_coords, IVec2::splat(UNIT_LENGTH));

        if let Some(WilloAnimationState::Push(direction)) = willo_animation_state {
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

fn perform_grid_coords_movement(
    mut grid_coords_query: Query<(
        Entity,
        &mut GridCoords,
        &RigidBody,
        Option<&mut WilloAnimationState>,
    )>,
    mut reader: EventReader<WilloMovementEvent>,
    level_query: Query<&Handle<LdtkLevel>>,
    levels: Res<Assets<LdtkLevel>>,
    audio: Res<Audio>,
    sfx: Res<AssetHolder>,
) {
    for movement_event in reader.iter() {
        let level = levels
            .get(level_query.single())
            .expect("Level should be loaded in graveyard state");

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

        if let Some((_, willo_grid_coords, _, Some(mut animation))) = grid_coords_query
            .iter_mut()
            .find(|(_, _, _, animation)| animation.is_some())
        {
            let pushed_entities = push_grid_coords_recursively(
                collision_map,
                IVec2::from(*willo_grid_coords),
                movement_event.direction,
            );

            if pushed_entities.len() > 1 {
                audio.play(sfx.push_sound.clone_weak());
                *animation.into_inner() = WilloAnimationState::Push(movement_event.direction);
            } else {
                let new_state = WilloAnimationState::Idle(movement_event.direction);
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
