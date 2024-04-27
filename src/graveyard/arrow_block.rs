use std::marker::PhantomData;

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
    #[asset(texture_atlas(tile_size_x = 64., tile_size_y = 64., columns = 4, rows = 4))]
    #[asset(path = "textures/movement-table-actions.png")]
    movement_tiles: Handle<TextureAtlas>,
}

trait Dimension {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Row;

impl Dimension for Row {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Column;

impl Dimension for Column {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Component)]
struct ArrowBlock<D>
where
    D: Dimension,
{
    direction: Direction,
    phantom_data: PhantomData<D>,
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
