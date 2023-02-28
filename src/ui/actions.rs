//! Plugin for associating UI buttons with logical actions.
//!
//! When a button with a [`UiAction<T>`](UiAction) component is clicked, the plugin will fire an
//! equivalent `UiAction<T>` event.
//! `T` can be any type that you want to use to distinguish between different buttons/actions.

use std::marker::PhantomData;

use crate::previous_component::{PreviousComponent, TrackPreviousComponent};
use bevy::prelude::*;

/// Label used for detecting UI interactions and firing UiAction events.
#[derive(SystemLabel)]
pub struct UiActionLabel;

/// Plugin for associating UI buttons with logical actions.
///
/// See [module-level docs](self) for more info.
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
    /// Basic constructor for [UiActionPlugin]
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

/// Component and event for associating a UI button with an action.
///
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
