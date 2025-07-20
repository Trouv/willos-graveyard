//! Common utilities that may be useful in many contexts.
use bevy::{ecs::query::QueryFilter, prelude::*};

/// Simple run condition that passes if the resource has changed.
pub fn resource_changed<R: Resource>(resource: Res<R>) -> bool {
    resource.is_changed()
}

/// Simple run condition that passes if any entities match the filter.
pub fn any_match_filter<F: QueryFilter>(filter_query: Query<(), F>) -> bool {
    !filter_query.is_empty()
}
