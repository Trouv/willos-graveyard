//! Utilities related to the graveyard's LDtk layers.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use thiserror::Error;

/// Layer entity does not exist.
#[derive(Debug, Error)]
#[error("{0:?} layer entity does not exist.")]
pub struct LayerEntityDoesNotExist(GraveyardLayer);

/// All LDtk layers in graveyard levels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GraveyardLayer {
    /// The background layer, usually for ground textures.
    Background,
    /// Another background layer, specifically for border textures that should appear behind willo.
    BorderBackground,
    /// Another background layer, but for entities that should appear behind willo.
    BackgroundEntities,
    /// Layer for gameplay tiles, like walls and volatile textures.
    IntGrid,
    /// Layer for sokoban entities, like willo and pushable blocks.
    Entities,
    /// Layer for things that appear infront of willo, like certain parts of border textures.
    Foreground,
}

impl GraveyardLayer {
    const BACKGROUND_IDENTIFIER: &'static str = "Background";
    const BORDER_BACKGROUND_IDENTIFIER: &'static str = "BorderBG";
    const BACKGROUND_ENTITIES_IDENTIFIER: &'static str = "Background_Entities";
    const INT_GRID_IDENTIFIER: &'static str = "IntGrid";
    const ENTITIES_IDENTIFIER: &'static str = "Entities";
    const FOREGROUND_IDENTIFIER: &'static str = "ForeGround";

    fn as_identifier(&self) -> &'static str {
        match self {
            GraveyardLayer::Background => Self::BACKGROUND_IDENTIFIER,
            GraveyardLayer::BorderBackground => Self::BORDER_BACKGROUND_IDENTIFIER,
            GraveyardLayer::BackgroundEntities => Self::BACKGROUND_ENTITIES_IDENTIFIER,
            GraveyardLayer::IntGrid => Self::INT_GRID_IDENTIFIER,
            GraveyardLayer::Entities => Self::ENTITIES_IDENTIFIER,
            GraveyardLayer::Foreground => Self::FOREGROUND_IDENTIFIER,
        }
    }

    /// Returns a system that takes a pipe input of bundles and spawns them on this layer.
    pub fn spawn_bundles_on<I>(
        self,
    ) -> impl Fn(In<I>, Commands, Query<(Entity, &LayerMetadata)>) -> Result<(), LayerEntityDoesNotExist>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Bundle,
    {
        move |In(bundles_iter), mut commands, layer_query| {
            let (layer_entity, _) = layer_query
                .iter()
                .find(|(_, metadata)| metadata.identifier == self.as_identifier())
                .ok_or(LayerEntityDoesNotExist(self))?;

            bundles_iter.into_iter().for_each(|bundle| {
                commands.spawn(bundle).set_parent(layer_entity);
            });

            Ok(())
        }
    }
}
