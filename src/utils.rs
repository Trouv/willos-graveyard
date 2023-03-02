use bevy::prelude::*;

pub fn resource_changed<R: Resource>(resource: Res<R>) -> bool {
    resource.is_changed()
}
