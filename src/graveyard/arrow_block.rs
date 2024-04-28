use std::{collections::HashMap, marker::PhantomData};

use bevy::{prelude::*, reflect::Enum, sprite::Anchor};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        config::{ConfigureLoadingState, LoadingStateConfig},
        LoadingStateAppExt,
    },
};
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};

use crate::{
    history::History,
    sokoban::{Direction, SokobanBlock},
    GameState, UNIT_LENGTH,
};

pub struct ArrowBlockPlugin;

impl Plugin for ArrowBlockPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading)
                .load_collection::<MovementTileAssets>(),
        )
        .register_ldtk_entity::<ArrowBluckBundle<Row>>("UpRow")
        .register_ldtk_entity::<ArrowBluckBundle<Row>>("LeftRow")
        .register_ldtk_entity::<ArrowBluckBundle<Row>>("DownRow")
        .register_ldtk_entity::<ArrowBluckBundle<Row>>("RightRow")
        .register_ldtk_entity::<ArrowBluckBundle<Column>>("UpColumn")
        .register_ldtk_entity::<ArrowBluckBundle<Column>>("LeftColumn")
        .register_ldtk_entity::<ArrowBluckBundle<Column>>("DownColumn")
        .register_ldtk_entity::<ArrowBluckBundle<Column>>("RightColumn");
    }
}

#[derive(Clone, Debug, AssetCollection, Resource)]
struct MovementTileAssets {
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 9, rows = 9))]
    #[asset(path = "textures/movement-table-actions.png")]
    movement_tiles: Handle<TextureAtlas>,
}

trait Dimension {
    fn significant_coordinate(grid_coords: &GridCoords) -> &i32;
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Row;

impl Dimension for Row {
    fn significant_coordinate(grid_coords: &GridCoords) -> &i32 {
        &grid_coords.y
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Column;

impl Dimension for Column {
    fn significant_coordinate(grid_coords: &GridCoords) -> &i32 {
        &grid_coords.x
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Component)]
struct ArrowBlock<D>
where
    D: Dimension,
{
    direction: Direction,
    phantom_data: PhantomData<D>,
}

impl<D> ArrowBlock<D>
where
    D: Dimension,
{
    fn fold_direction_into(
        &self,
        self_grid_coords: &GridCoords,
        mut aggregate_directions: HashMap<i32, Direction>,
    ) -> HashMap<i32, Direction> {
        *aggregate_directions
            .entry(*D::significant_coordinate(self_grid_coords))
            .or_default() += self.direction;

        aggregate_directions
    }
}

impl<D> From<&EntityInstance> for ArrowBlock<D>
where
    D: Dimension,
{
    fn from(value: &EntityInstance) -> Self {
        let direction = if value.identifier.contains("Up") {
            Direction::Up
        } else if value.identifier.contains("Left") {
            Direction::Left
        } else if value.identifier.contains("Down") {
            Direction::Down
        } else if value.identifier.contains("Right") {
            Direction::Right
        } else {
            panic!("ArrowBlock Identifier should contain direction")
        };

        Self {
            direction,
            phantom_data: PhantomData,
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct ArrowBluckBundle<D>
where
    D: Dimension + Send + Sync + Default + 'static,
{
    #[from_entity_instance]
    arrow_block: ArrowBlock<D>,
    #[grid_coords]
    grid_coords: GridCoords,
    history: History<GridCoords>,
    #[with(SokobanBlock::new_dynamic)]
    sokoban_block: SokobanBlock,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Component)]
struct MovementTile {
    row_move: Direction,
    column_move: Direction,
}

#[derive(Clone, Default, Bundle)]
struct MovementTileBundle {
    grid_coords: GridCoords,
    movement_tile: MovementTile,
    sprite_sheet_bundle: SpriteSheetBundle,
}

impl MovementTileBundle {
    fn new(
        grid_coords: GridCoords,
        movement_tile: MovementTile,
        movement_tile_assets: &MovementTileAssets,
    ) -> Self {
        let translation =
            grid_coords_to_translation(grid_coords, IVec2::splat(UNIT_LENGTH)).extend(0.);

        let scale = Vec3::new(0.5, 0.5, 1.);

        let transform = Transform::from_translation(translation).with_scale(scale);

        let index =
            movement_tile.row_move.variant_index() * 9 + movement_tile.column_move.variant_index();

        let sprite = TextureAtlasSprite { index, ..default() };
        let sprite_sheet_bundle = SpriteSheetBundle {
            sprite,
            texture_atlas: movement_tile_assets.movement_tiles.clone(),
            transform,
            ..default()
        };

        MovementTileBundle {
            grid_coords,
            movement_tile,
            sprite_sheet_bundle,
        }
    }
}

fn despawn_movement_tiles(
    mut commands: Commands,
    current_movement_tiles: Query<Entity, With<MovementTile>>,
) {
    current_movement_tiles.for_each(|entity| commands.entity(entity).despawn_recursive());
}

struct MovementTileBundleIntoIter<'w> {
    movement_tile_assets: &'w MovementTileAssets,
    aggregate_row_directions: HashMap<i32, Direction>,
    aggregate_column_directions: HashMap<i32, Direction>,
}

/// Should be run with conservative run criteria
fn all_movement_tiles_at_intersections<'w>(
    row_blocks: Query<(&GridCoords, &ArrowBlock<Row>)>,
    column_blocks: Query<(&GridCoords, &ArrowBlock<Column>)>,
    movement_tile_assets: Res<'w, MovementTileAssets>,
) -> Vec<MovementTileBundle> {
    let aggregate_row_directions =
        row_blocks
            .iter()
            .fold(default(), |aggregate, (grid_coords, arrow_block)| {
                arrow_block.fold_direction_into(grid_coords, aggregate)
            });

    let aggregate_column_directions =
        column_blocks
            .iter()
            .fold(default(), |aggregate, (grid_coords, arrow_block)| {
                arrow_block.fold_direction_into(grid_coords, aggregate)
            });

    aggregate_row_directions
        .into_iter()
        .flat_map(move |row_item| {
            std::iter::repeat(row_item).zip(aggregate_column_directions.clone())
        })
        .map(move |((y, row_move), (x, column_move))| {
            let grid_coords = GridCoords::new(x, y);
            let movement_tile = MovementTile {
                row_move,
                column_move,
            };

            MovementTileBundle::new(grid_coords, movement_tile, &movement_tile_assets)
        })
        .collect()
}
