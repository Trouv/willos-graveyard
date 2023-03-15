//! Plugin for tracking the previous value of a component via an additional generic component.
//!
//! Similar to the [crate::history] API, but keeps track of only one change.
use bevy::prelude::*;
use std::marker::PhantomData;

/// System label used for updating the [PreviousComponent] value.
/// Consider placing your system after this if you are depending on an accurate [PreviousComponent].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemSet)]
pub struct TrackPreviousComponent;

/// Plugin for tracking the previous value of a component via an additional generic component.
///
/// You'll need to insert this plugin to the app multiple times for every component you want to track.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct PreviousComponentPlugin<C: Component> {
    phantom: PhantomData<C>,
}

impl<C: Component + Clone> Plugin for PreviousComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_system(track_previous_component::<C>.label(TrackPreviousComponent));
    }
}

/// Component for tracking the previous value of another component on the same entity, `C`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct PreviousComponent<C: Component> {
    current: C,
    last: C,
}

impl<C: Component> PreviousComponent<C> {
    /// Gets the previous value for the tracked component `C` on this entity.
    pub fn get(&self) -> &C {
        &self.last
    }
}

/// System for updating [PreviousComponent] values.
fn track_previous_component<C: Component + Clone>(
    mut components: Query<(&C, &mut PreviousComponent<C>), Changed<C>>,
) {
    for (component, mut previous) in components.iter_mut() {
        previous.last = previous.current.clone();
        previous.current = component.clone();
    }
}
