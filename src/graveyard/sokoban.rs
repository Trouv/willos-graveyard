//! Plugin and components providing functionality for sokoban-style movement and collision.
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_easings::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use iyes_loopless::prelude::*;
use std::any::Any;

/// Labels used by sokoban systems
#[derive(SystemLabel)]
pub enum SokobanLabels {
    /// Label for the system that updates the visual position of sokoban entities via bevy_easings.
    EaseMovement,
    /// Label for the system that updates the logical position of sokoban entities.
    LogicalMovement,
}

/// Plugin providing functionality for sokoban-style movement and collision.
pub struct SokobanPlugin<S> {
    state: S,
    layer_identifier: SokobanLayerIdentifier,
}

impl<S> SokobanPlugin<S> {
    /// Constructor for the plugin.
    ///
    /// Allows the user to specify a particular iyes_loopless state to run the plugin in.
    ///
    /// The `layer_identifier` should refer to a non-entity layer in LDtk that can be treated as
    /// the Sokoban grid.
    /// This layer should have the tile-size and dimensions for your desired sokoban functionality.
    pub fn new(state: S, layer_identifier: impl Into<String>) -> Self {
        let layer_identifier = SokobanLayerIdentifier(layer_identifier.into());
        SokobanPlugin {
            state,
            layer_identifier,
        }
    }
}

impl<S> Plugin for SokobanPlugin<S>
where
    S: Any + Send + Sync + Clone + std::fmt::Debug + std::hash::Hash + Eq,
{
    fn build(&self, app: &mut App) {
        app.add_event::<SokobanCommand>()
            .add_event::<PushEvent>()
            .insert_resource(self.layer_identifier.clone())
            .add_system(
                flush_sokoban_commands
                    .run_in_state(self.state.clone())
                    .run_on_event::<SokobanCommand>()
                    .label(SokobanLabels::LogicalMovement),
            )
            // Systems with potential easing end/beginning collisions cannot be in CoreStage::Update
            // see https://github.com/vleue/bevy_easings/issues/23
            .add_system_to_stage(
                CoreStage::PostUpdate,
                ease_movement
                    .run_in_state(self.state.clone())
                    .label(SokobanLabels::EaseMovement),
            )
            .register_ldtk_int_cell::<WallBundle>(1)
            .register_ldtk_int_cell::<WallBundle>(3)
            .register_ldtk_int_cell::<WallBundle>(4);
    }
}

/// Resource referring to the LDtk layer that should be treated as a sokoban grid.
#[derive(Debug, Clone, Deref, DerefMut, Resource)]
struct SokobanLayerIdentifier(String);

/// Enumerates the four directions that sokoban blocks can be pushed in.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Direction {
    /// North direction.
    Up,
    /// West direction.
    Left,
    /// South direction.
    Down,
    /// East direction.
    Right,
}

impl From<Direction> for IVec2 {
    fn from(direction: Direction) -> IVec2 {
        match direction {
            Direction::Up => IVec2::Y,
            Direction::Left => IVec2::new(-1, 0),
            Direction::Down => IVec2::new(0, -1),
            Direction::Right => IVec2::X,
        }
    }
}

/// Enumerates commands that can be performed via [SokobanCommands].
#[derive(Debug, Clone)]
pub enum SokobanCommand {
    /// Move a [SokobanBlock] entity in the given direction.
    Move {
        /// The [SokobanBlock] entity to move.
        entity: Entity,
        /// The direction to move the block in.
        direction: Direction,
    },
}

/// System parameter providing an interface for commanding the SokobanPlugin.
#[derive(SystemParam)]
pub struct SokobanCommands<'w, 's> {
    writer: EventWriter<'w, 's, SokobanCommand>,
}

impl<'w, 's> SokobanCommands<'w, 's> {
    /// Move a [SokobanBlock] entity in the given direction.
    ///
    /// Will perform the necessary collision checks and block pushes.
    pub fn move_block(&mut self, entity: Entity, direction: Direction) {
        self.writer.send(SokobanCommand::Move { entity, direction });
    }
}

/// Component defining the behavior of sokoban entities on collision.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum SokobanBlock {
    /// The entity cannot move, push, or be pushed - but can block movement.
    Static,
    /// The entity can move, push, or be pushed.
    Dynamic,
}

/// Component that marks [SokobanBlock]s that should fire [PushEvent]s when they push other blocks.
#[derive(Debug, Component)]
pub struct PushTracker;

/// Event that fires when a [PushTracker] entity pushes other [SokobanBlock]s.
#[derive(Debug, Clone)]
pub struct PushEvent {
    /// The [PushTracker] entity that pushed other [SokobanBlock]s.
    pub pusher: Entity,
    /// The direction of the push.
    pub direction: Direction,
    /// The list of [SokobanBlock] entities that were pushed.
    pub pushed: Vec<Entity>,
}

#[derive(Clone, Bundle, LdtkIntCell)]
struct WallBundle {
    #[from_int_grid_cell]
    sokoban_block: SokobanBlock,
}

impl From<EntityInstance> for SokobanBlock {
    fn from(_: EntityInstance) -> SokobanBlock {
        SokobanBlock::Dynamic
    }
}

impl From<IntGridCell> for SokobanBlock {
    fn from(_: IntGridCell) -> SokobanBlock {
        SokobanBlock::Static
    }
}

fn ease_movement(
    mut commands: Commands,
    mut grid_coords_query: Query<
        (Entity, &GridCoords, &Transform),
        (Changed<GridCoords>, With<SokobanBlock>),
    >,
    layers: Query<&LayerMetadata>,
    layer_id: Res<SokobanLayerIdentifier>,
) {
    for (entity, &grid_coords, transform) in grid_coords_query.iter_mut() {
        if let Some(LayerMetadata { grid_size, .. }) =
            layers.iter().find(|l| l.identifier == **layer_id)
        {
            let xy = grid_coords_to_translation(grid_coords, IVec2::splat(*grid_size));

            commands.entity(entity).insert(transform.ease_to(
                Transform::from_xyz(xy.x, xy.y, transform.translation.z),
                EaseFunction::CubicOut,
                EasingType::Once {
                    duration: std::time::Duration::from_millis(110),
                },
            ));
        }
    }
}

type CollisionMap = Vec<Vec<Option<(Entity, SokobanBlock)>>>;

/// Pushes the entry at the given coordinates in the collision_map in the given direction.
///
/// If possible, it will also push any entries it collides with.
///
/// # Returns
/// Returns a tuple containing the updated collision_map, and an optional list of pushed entities.
///
/// If the optional list is `None`, no entities were pushed due to collision with either a
/// [SokobanBlock::Static] entry or a boundary of the map.
///
/// If the optional list is empty, no entities were pushed due to the provided coordinates pointing
/// to an empty entry. This distinction is important for the recursive algorithm.
fn push_grid_coords_recursively(
    collision_map: CollisionMap,
    pusher_coords: IVec2,
    direction: Direction,
) -> (CollisionMap, Option<Vec<Entity>>) {
    // check if pusher is out-of-bounds
    if pusher_coords.x < 0
        || pusher_coords.y < 0
        || pusher_coords.y as usize >= collision_map.len()
        || pusher_coords.x as usize >= collision_map[0].len()
    {
        // no updates to collision_map, no pushes can be performed
        return (collision_map, None);
    }

    // match against the pusher's CollisionMap entry
    match collision_map[pusher_coords.y as usize][pusher_coords.x as usize] {
        Some((pusher, SokobanBlock::Dynamic)) => {
            // pusher is dynamic, so we try to push
            let destination = IVec2::from(pusher_coords) + IVec2::from(direction);

            match push_grid_coords_recursively(collision_map, destination, direction) {
                (mut collision_map, Some(mut pushed_entities)) => {
                    // destination is either empty or has been pushed, so we can push the pusher
                    collision_map[destination.y as usize][destination.x as usize] =
                        collision_map[pusher_coords.y as usize][pusher_coords.x as usize].take();
                    pushed_entities.push(pusher);

                    (collision_map, Some(pushed_entities))
                }
                // destination can't be pushed, so the pusher can't be pushed either
                none_case => none_case,
            }
        }
        // pusher is static, no pushes can be performed
        Some((_, SokobanBlock::Static)) => (collision_map, None),
        // pusher's entry is empty, no push is performed here but the caller is able to
        None => (collision_map, Some(Vec::new())),
    }
}

fn flush_sokoban_commands(
    mut grid_coords_query: Query<(Entity, &mut GridCoords, &SokobanBlock, Option<&PushTracker>)>,
    mut sokoban_commands: EventReader<SokobanCommand>,
    mut push_events: EventWriter<PushEvent>,
    layers: Query<&LayerMetadata>,
    layer_id: Res<SokobanLayerIdentifier>,
) {
    // Get dimensions of the currently-loaded level
    if let Some(LayerMetadata { c_wid, c_hei, .. }) =
        layers.iter().find(|l| l.identifier == **layer_id)
    {
        // Generate current collision map
        let mut collision_map: CollisionMap = vec![vec![None; *c_wid as usize]; *c_hei as usize];

        for (entity, grid_coords, sokoban_block, _) in grid_coords_query.iter_mut() {
            collision_map[grid_coords.y as usize][grid_coords.x as usize] =
                Some((entity, *sokoban_block));
        }

        for sokoban_command in sokoban_commands.iter() {
            let SokobanCommand::Move { entity, direction } = sokoban_command;

            if let Ok((_, grid_coords, ..)) = grid_coords_query.get(*entity) {
                // Determine if move can happen, who moves, how the collision_map should be
                // updated...
                let (new_collision_map, pushed_entities) = push_grid_coords_recursively(
                    collision_map,
                    IVec2::from(*grid_coords),
                    *direction,
                );

                collision_map = new_collision_map;

                if let Some(mut pushed_entities) = pushed_entities {
                    pushed_entities.reverse();

                    // update GridCoords components of pushed entities
                    for pushed_entity in &pushed_entities {
                        *grid_coords_query
                            .get_component_mut::<GridCoords>(*pushed_entity)
                            .expect("pushed entity should have GridCoords component") +=
                            GridCoords::from(IVec2::from(*direction));
                    }

                    // send push events
                    for (i, pusher) in pushed_entities.iter().enumerate() {
                        let pushed = &pushed_entities[i + 1..];

                        if pushed.len() > 1 {
                            if let Ok((.., Some(_))) = grid_coords_query.get(*pusher) {
                                push_events.send(PushEvent {
                                    pusher: *pusher,
                                    direction: *direction,
                                    pushed: pushed.into(),
                                });
                            }
                        }
                    }
                }
            }
        }
    } else {
        warn!(
            "could not find {} layer specified by SokobanLayerIdentifier resource",
            **layer_id
        );
    }
}
