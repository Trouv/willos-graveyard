//! Contains [UiAction] and related systems.

use std::marker::PhantomData;

use crate::previous_component::{PreviousComponent, TrackPreviousComponent};
use bevy::prelude::*;

#[derive(SystemLabel)]
pub struct UiActionLabel;

#[derive(Default)]
pub struct UiActionPlugin<T>
where
    T: Send + Sync + Clone + 'static,
{
    phantom_data: PhantomData<T>,
}

impl<T> UiActionPlugin<T>
where
    T: Send + Sync + Clone + 'static,
{
    pub fn new() -> UiActionPlugin<T> {
        UiActionPlugin {
            phantom_data: PhantomData,
        }
    }
}

impl<T> Plugin for UiActionPlugin<T>
where
    T: Send + Sync + Clone + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_event::<UiAction<T>>().add_system(
            ui_action::<T>
                .label(UiActionLabel)
                .after(TrackPreviousComponent),
        );
    }
}

/// All possible actions that can be triggered by the UI.
///
/// This acts as both a component and an event.
/// Insert it on a button to define what action that button performs.
/// Then, when that button is pressed, an event of the same value will be fired.
#[allow(dead_code)]
#[derive(Clone, Eq, PartialEq, Debug, Default, Deref, DerefMut, Component)]
pub struct UiAction<T: Send + Sync + Clone + 'static>(pub T);

/// System that detects button presses and fires [UiAction]s.
pub(super) fn ui_action<T>(
    actions: Query<
        (&UiAction<T>, &Interaction, &PreviousComponent<Interaction>),
        Changed<Interaction>,
    >,
    mut event_writer: EventWriter<UiAction<T>>,
) where
    T: Send + Sync + Clone + 'static,
{
    for (action, interaction, previous) in actions.iter() {
        if (Interaction::Hovered, Interaction::Clicked) == (*interaction, *previous.get()) {
            event_writer.send(action.clone())
        }
    }
}
