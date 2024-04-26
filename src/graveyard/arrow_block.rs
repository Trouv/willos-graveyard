use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    history::History,
    sokoban::{Direction, SokobanBlock},
};

pub struct ArrowBlockPlugin;

impl Plugin for ArrowBlockPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<ArrowBluckBundle<Row>>("UpRow")
            .register_ldtk_entity::<ArrowBluckBundle<Row>>("LeftRow")
            .register_ldtk_entity::<ArrowBluckBundle<Row>>("DownRow")
            .register_ldtk_entity::<ArrowBluckBundle<Row>>("RightRow")
            .register_ldtk_entity::<ArrowBluckBundle<Column>>("UpColumn")
            .register_ldtk_entity::<ArrowBluckBundle<Column>>("LeftColumn")
            .register_ldtk_entity::<ArrowBluckBundle<Column>>("DownColumn")
            .register_ldtk_entity::<ArrowBluckBundle<Column>>("RightColumn");
    }
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
