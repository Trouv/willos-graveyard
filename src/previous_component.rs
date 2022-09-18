use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemLabel)]
pub struct TrackPreviousComponent;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct PreviousComponentPlugin<C: Component> {
    phantom: PhantomData<C>,
}

impl<C: Component + Clone> Plugin for PreviousComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_system(track_previous_component::<C>.label(TrackPreviousComponent));
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct PreviousComponent<C: Component> {
    current: C,
    last: C,
}

impl<C: Component> PreviousComponent<C> {
    pub fn get(&self) -> &C {
        &self.last
    }
}

fn track_previous_component<C: Component + Clone>(
    mut components: Query<(&C, &mut PreviousComponent<C>), Changed<C>>,
) {
    for (component, mut previous) in components.iter_mut() {
        previous.last = previous.current.clone();
        previous.current = component.clone();
    }
}
