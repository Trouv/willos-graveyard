//! Plugin providing functionality for sokoban-style movement and collision to LDtk levels.
//!
//! You should use `bevy_ecs_ldtk` to load the levels.
//! Spawn entities with `GridCoords` (from `bevy_ecs_ldtk`) and [SokobanBlock]s to give them
//! sokoban-style collision.
//! Then, move entities around with the [SokobanCommands] system parameter.
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_easings::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};

/// Sets used by sokoban systems
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum SokobanSets {
    /// Set for the system that updates the visual position of sokoban entities via bevy_easings.
    EaseMovement,
    /// Set for the system that updates the logical position of sokoban entities.
    LogicalMovement,
}

/// Plugin providing functionality for sokoban-style movement and collision to LDtk levels.
pub struct SokobanPlugin<S>
where
    S: States,
{
    state: S,
    layer_identifier: SokobanLayerIdentifier,
}

impl<S> SokobanPlugin<S>
where
    S: States,
{
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
    S: States,
{
    fn build(&self, app: &mut App) {
        app.add_event::<SokobanCommand>()
            .add_event::<PushEvent>()
            .insert_resource(self.layer_identifier.clone())
            .add_systems(
                Update,
                flush_sokoban_commands
                    .run_if(in_state(self.state.clone()))
                    .run_if(on_event::<SokobanCommand>())
                    .in_set(SokobanSets::LogicalMovement),
            )
            // Systems with potential easing end/beginning collisions cannot be in CoreSet::Update
            // see https://github.com/vleue/bevy_easings/issues/23
            .add_systems(
                PostUpdate,
                ease_movement
                    .run_if(in_state(self.state.clone()))
                    .in_set(SokobanSets::EaseMovement),
            );
    }
}

/// Resource referring to the LDtk layer that should be treated as a sokoban grid.
#[derive(Debug, Clone, Deref, DerefMut, Resource)]
struct SokobanLayerIdentifier(String);

/// Enumerates the four directions that sokoban blocks can be pushed in.
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug, Hash)]
pub enum Direction {
    /// North direction.
    #[default]
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
#[derive(Debug, Clone, Event)]
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
pub struct SokobanCommands<'w> {
    writer: EventWriter<'w, SokobanCommand>,
}

impl SokobanCommands<'_> {
    /// Move a [SokobanBlock] entity in the given direction.
    ///
    /// Will perform the necessary collision checks and block pushes.
    pub fn move_block(&mut self, entity: Entity, direction: Direction) {
        self.writer.send(SokobanCommand::Move { entity, direction });
    }
}

/// Component defining the behavior of sokoban entities on collision.
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug, Hash, Component)]
pub enum SokobanBlock {
    #[default]
    /// The entity cannot move, push, or be pushed - but can block movement.
    Static,
    /// The entity can move, push, or be pushed.
    Dynamic,
}

impl SokobanBlock {
    /// Constructor returning [SokobanBlock::Static].
    ///
    /// Compatible with the `with` attribute for `#[derive(LdtkEntity)]`:
    /// ```
    /// use bevy_ecs_ldtk::*;
    ///
    /// #[derive(Bundle, LdtkEntity)]
    /// struct MyLdtkEntity {
    ///     #[grid_coords]
    ///     grid_coords: GridCoords,
    ///     #[with(SokobanBlock::new_static)]
    ///     sokoban_block: SokobanBlock,
    /// }
    /// ```
    pub fn new_static(_: &EntityInstance) -> SokobanBlock {
        SokobanBlock::Static
    }

    /// Constructor returning [SokobanBlock::Dynamic].
    ///
    /// Compatible with the `with` attribute for `#[derive(LdtkEntity)]`:
    /// ```
    /// use bevy_ecs_ldtk::*;
    ///
    /// #[derive(Bundle, LdtkEntity)]
    /// struct MyLdtkEntity {
    ///     #[grid_coords]
    ///     grid_coords: GridCoords,
    ///     #[with(SokobanBlock::new_dynamic)]
    ///     sokoban_block: SokobanBlock,
    /// }
    /// ```
    pub fn new_dynamic(_: &EntityInstance) -> SokobanBlock {
        SokobanBlock::Dynamic
    }
}

/// Component that marks [SokobanBlock]s that should fire [PushEvent]s when they push other blocks.
#[derive(Clone, Default, Debug, Component)]
pub struct PushTracker;

/// Event that fires when a [PushTracker] entity pushes other [SokobanBlock]s.
#[derive(Debug, Clone, PartialEq, Eq, Event)]
pub struct PushEvent {
    /// The [PushTracker] entity that pushed other [SokobanBlock]s.
    pub pusher: Entity,
    /// The direction of the push.
    pub direction: Direction,
    /// The list of [SokobanBlock] entities that were pushed.
    pub pushed: Vec<Entity>,
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
fn push_collision_map_entry(
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
            let destination = pusher_coords + IVec2::from(direction);

            match push_collision_map_entry(collision_map, destination, direction) {
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

        for sokoban_command in sokoban_commands.read() {
            let SokobanCommand::Move { entity, direction } = sokoban_command;

            if let Ok((_, grid_coords, ..)) = grid_coords_query.get(*entity) {
                // Determine if move can happen, who moves, how the collision_map should be
                // updated...
                let (new_collision_map, pushed_entities) =
                    push_collision_map_entry(collision_map, IVec2::from(*grid_coords), *direction);

                collision_map = new_collision_map;

                if let Some(mut pushed_entities) = pushed_entities {
                    pushed_entities.reverse();

                    // update GridCoords components of pushed entities
                    for pushed_entity in &pushed_entities {
                        *grid_coords_query
                            .get_component_mut::<GridCoords>(*pushed_entity)
                            .expect("pushed entity should be valid sokoban entity") +=
                            GridCoords::from(IVec2::from(*direction));
                    }

                    // send push events
                    for (i, pusher) in pushed_entities.iter().enumerate() {
                        let pushed = &pushed_entities[i + 1..];

                        if !pushed.is_empty() {
                            if let (.., Some(_)) = grid_coords_query
                                .get(*pusher)
                                .expect("pusher should be valid sokoban entity")
                            {
                                push_events.send(PushEvent {
                                    pusher: *pusher,
                                    direction: *direction,
                                    pushed: pushed.into(),
                                });
                            }
                        }
                    }
                }
            } else {
                warn!("attempted to move sokoban entity {entity:?}, but it does not exist or is malformed")
            }
        }
    } else {
        warn!(
            "could not find {} layer specified by SokobanLayerIdentifier resource",
            **layer_id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn push_dynamic_into_empty() {
        let pusher = Entity::from_raw(0);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[1][1] = Some((pusher, SokobanBlock::Dynamic));

        let mut expected_collision_map = vec![vec![None; 3]; 3];
        expected_collision_map[2][1] = Some((pusher, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(collision_map, IVec2::new(1, 1), super::Direction::Up),
            (expected_collision_map, Some(vec![pusher]))
        );
    }

    #[test]
    fn push_dynamic_into_static() {
        let pusher = Entity::from_raw(0);
        let wall = Entity::from_raw(1);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[2][1] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[1][1] = Some((wall, SokobanBlock::Static));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(1, 2),
                super::Direction::Down
            ),
            (collision_map, None)
        );
    }

    #[test]
    fn push_dynamic_into_boundary() {
        let pusher = Entity::from_raw(0);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[0][0] = Some((pusher, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(0, 0),
                super::Direction::Left
            ),
            (collision_map, None)
        );
    }

    #[test]
    fn push_dynamic_into_dynamic_into_empty() {
        let pusher = Entity::from_raw(0);
        let pushed = Entity::from_raw(1);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[1][0] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[1][1] = Some((pushed, SokobanBlock::Dynamic));

        let mut expected_collision_map = vec![vec![None; 3]; 3];
        expected_collision_map[1][1] = Some((pusher, SokobanBlock::Dynamic));
        expected_collision_map[1][2] = Some((pushed, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(collision_map, IVec2::new(0, 1), super::Direction::Right),
            (expected_collision_map, Some(vec![pushed, pusher]))
        );
    }

    #[test]
    fn push_dynamic_into_dynamic_into_static() {
        let pusher = Entity::from_raw(0);
        let pushed = Entity::from_raw(1);
        let wall = Entity::from_raw(2);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[2][0] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[2][1] = Some((pushed, SokobanBlock::Dynamic));
        collision_map[2][2] = Some((wall, SokobanBlock::Static));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(0, 2),
                super::Direction::Right
            ),
            (collision_map, None)
        );
    }

    #[test]
    fn push_dynamic_into_dynamic_into_boundary() {
        let pusher = Entity::from_raw(0);
        let pushed = Entity::from_raw(1);

        let mut collision_map = vec![vec![None; 3]; 3];
        collision_map[1][1] = Some((pusher, SokobanBlock::Dynamic));
        collision_map[2][1] = Some((pushed, SokobanBlock::Dynamic));

        assert_eq!(
            push_collision_map_entry(
                collision_map.clone(),
                IVec2::new(1, 1),
                super::Direction::Up,
            ),
            (collision_map, None)
        );
    }

    fn app_setup() -> App {
        #[derive(Clone, PartialEq, Eq, Debug, Default, Hash, States)]
        enum State {
            #[default]
            Only,
        }

        let mut app = App::new();

        app.add_state::<State>()
            .add_plugins(SokobanPlugin::new(State::Only, "MyLayerIdentifier"));

        app.world.spawn(LayerMetadata {
            c_wid: 3,
            c_hei: 4,
            grid_size: 32,
            identifier: "MyLayerIdentifier".to_string(),
            ..default()
        });

        app
    }

    #[test]
    fn push_with_sokoban_commands() {
        let mut app = app_setup();

        let block_a = app
            .world
            .spawn((GridCoords::new(1, 1), SokobanBlock::Dynamic))
            .id();
        let block_b = app
            .world
            .spawn((GridCoords::new(1, 2), SokobanBlock::Dynamic))
            .id();
        let block_c = app
            .world
            .spawn((GridCoords::new(2, 2), SokobanBlock::Dynamic))
            .id();

        let mut system_state: SystemState<SokobanCommands> = SystemState::new(&mut app.world);
        let mut sokoban_commands: SokobanCommands = system_state.get_mut(&mut app.world);

        sokoban_commands.move_block(block_a, super::Direction::Up);
        sokoban_commands.move_block(block_c, super::Direction::Left);

        system_state.apply(&mut app.world);

        app.update();

        assert_eq!(
            *app.world.entity(block_a).get::<GridCoords>().unwrap(),
            GridCoords::new(0, 2)
        );
        assert_eq!(
            *app.world.entity(block_b).get::<GridCoords>().unwrap(),
            GridCoords::new(1, 3)
        );
        assert_eq!(
            *app.world.entity(block_c).get::<GridCoords>().unwrap(),
            GridCoords::new(1, 2)
        );
    }

    #[test]
    fn push_tracker_sends_events() {
        let mut app = app_setup();

        let block_a = app
            .world
            .spawn((GridCoords::new(1, 1), SokobanBlock::Dynamic, PushTracker))
            .id();
        let block_b = app
            .world
            .spawn((GridCoords::new(1, 2), SokobanBlock::Dynamic))
            .id();
        let block_c = app
            .world
            .spawn((GridCoords::new(2, 2), SokobanBlock::Dynamic))
            .id();

        let mut system_state: SystemState<SokobanCommands> = SystemState::new(&mut app.world);
        let mut sokoban_commands: SokobanCommands = system_state.get_mut(&mut app.world);

        sokoban_commands.move_block(block_a, super::Direction::Up);
        sokoban_commands.move_block(block_c, super::Direction::Left);

        system_state.apply(&mut app.world);

        app.update();

        let events = app.world.resource::<Events<PushEvent>>();
        let mut reader = events.get_reader();

        assert_eq!(events.len(), 1);
        assert_eq!(
            *reader.read(events).next().unwrap(),
            PushEvent {
                pusher: block_a,
                direction: super::Direction::Up,
                pushed: vec![block_b],
            }
        );
    }
}
