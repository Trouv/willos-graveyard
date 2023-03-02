//! Common utilities that may be useful in many contexts.
use bevy::prelude::*;

/// Simple `iyes_loopless` run condition that passes if the resource has changed.
pub fn resource_changed<R: Resource>(resource: Res<R>) -> bool {
    resource.is_changed()
}
