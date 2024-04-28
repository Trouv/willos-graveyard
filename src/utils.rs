//! Common utilities that may be useful in many contexts.
use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*};
use bevy_ecs_ldtk::prelude::*;

/// Simple run condition that passes if the resource has changed.
pub fn resource_changed<R: Resource>(resource: Res<R>) -> bool {
    resource.is_changed()
}

/// Simple run condition that passes if any entities match the filter.
pub fn any_match_filter<F: ReadOnlyWorldQuery>(filter_query: Query<(), F>) -> bool {
    !filter_query.is_empty()
}

const BACKGROUND_ENTITIES_LAYER_IDENTIFIER: &str = "Background_Entities";

pub fn spawn_on_background_entities_layer<I>(
    In(bundles_iter): In<I>,
    mut commands: Commands,
    layer_query: Query<(Entity, &LayerMetadata)>,
) where
    I: IntoIterator,
    <I as IntoIterator>::Item: Bundle,
{
    let (background_entity_layer, _) = layer_query
        .iter()
        .find(|(_, metadata)| metadata.identifier == BACKGROUND_ENTITIES_LAYER_IDENTIFIER)
        .expect("attempted to spawn entities on {} layer that does not exist");

    bundles_iter.into_iter().for_each(|bundle| {
        commands.spawn(bundle).set_parent(background_entity_layer);
    });
}
