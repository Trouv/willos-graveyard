//! Plugin and components providing functionality for sokoban-style movement and collision.
use crate::{
    from_component::FromComponentLabel,
    graveyard::{
        movement_table::{Direction, MovementTable},
        willo::WilloAnimationState,
    },
    GameState, UNIT_LENGTH,
};
use bevy::{ecs::system::SystemParam, prelude::*};
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
pub struct SokobanPlugin {
    layer_identifier: SokobanLayerIdentifier,
}

impl SokobanPlugin {
    pub fn new(layer_identifier: impl Into<String>) -> Self {
        let layer_identifier = SokobanLayerIdentifier(layer_identifier.into());
        SokobanPlugin { layer_identifier }
    }
}

impl Plugin for SokobanPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SokobanCommand>()
            .add_event::<PushEvent>()
            .insert_resource(self.layer_identifier.clone())
            .add_system(
                flush_sokoban_commands
                    .run_in_state(GameState::Graveyard)
                    .run_on_event::<SokobanCommand>()
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

#[derive(Debug, Clone, Deref, DerefMut, Resource)]
pub struct SokobanLayerIdentifier(String);

#[derive(Debug, Clone)]
pub enum SokobanCommand {
    Move {
        entity: Entity,
        direction: Direction,
    },
}

#[derive(SystemParam)]
pub struct SokobanCommands<'w, 's> {
    writer: EventWriter<'w, 's, SokobanCommand>,
}

impl<'w, 's> SokobanCommands<'w, 's> {
    pub fn move_entity(&mut self, entity: Entity, direction: Direction) {
        self.writer.send(SokobanCommand::Move { entity, direction });
    }
}

#[derive(Debug, Clone)]
pub struct PushEvent {
    pub entity: Entity,
    pub direction: Direction,
    pub pushed: Vec<Entity>,
}

/// Component defining the behavior of sokoban entities on collision.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum SokobanBlock {
    Static,
    Dynamic,
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
    collision_map: Vec<Vec<Option<(Entity, SokobanBlock)>>>,
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
        Some((_, SokobanBlock::Static)) => Vec::new(),
        Some((_, SokobanBlock::Dynamic)) => {
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

fn flush_sokoban_commands(
    mut grid_coords_query: Query<(Entity, &mut GridCoords, &SokobanBlock)>,
    mut sokoban_commands: EventReader<SokobanCommand>,
    mut push_events: EventWriter<PushEvent>,
    layers: Query<&LayerMetadata>,
    layer_id: Res<SokobanLayerIdentifier>,
) {
    for sokoban_command in sokoban_commands.iter() {
        // Get dimensions of current level
        if let Some(LayerMetadata { c_wid, c_hei, .. }) =
            layers.iter().find(|l| l.identifier == **layer_id)
        {
            let SokobanCommand::Move { entity, direction } = sokoban_command;

            let mut collision_map: Vec<Vec<Option<(Entity, SokobanBlock)>>> =
                vec![vec![None; *c_wid as usize]; *c_hei as usize];

            for (entity, grid_coords, sokoban_block) in grid_coords_query.iter_mut() {
                collision_map[grid_coords.y as usize][grid_coords.x as usize] =
                    Some((entity, *sokoban_block));
            }

            if let Ok((_, grid_coords, _)) = grid_coords_query.get(*entity) {
                let pushed = push_grid_coords_recursively(
                    collision_map,
                    IVec2::from(*grid_coords),
                    *direction,
                );

                for pushed_entity in &pushed {
                    *grid_coords_query
                        .get_component_mut::<GridCoords>(*pushed_entity)
                        .expect("pushed entity should have GridCoords component") +=
                        GridCoords::from(IVec2::from(*direction));
                }

                if pushed.len() > 1 {
                    push_events.send(PushEvent {
                        entity: *entity,
                        direction: *direction,
                        pushed,
                    });
                }
            }
        }
    }
}
