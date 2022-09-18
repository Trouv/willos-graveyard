//! Contains [UiAction] and related systems.

use crate::previous_component::PreviousComponent;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

/// All possible actions that can be triggered by the UI.
///
/// This acts as both a component and an event.
/// Insert it on a button to define what action that button performs.
/// Then, when that button is pressed, an event of the same value will be fired.
#[allow(dead_code)]
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub enum UiAction {
    Debug(&'static str),
    RestartLevel,
    NextLevel,
    GoToLevel(LevelSelection),
}

/// System that detects button presses and fires [UiAction]s.
pub(super) fn ui_action(
    actions: Query<
        (&UiAction, &Interaction, &PreviousComponent<Interaction>),
        Changed<Interaction>,
    >,
    mut event_writer: EventWriter<UiAction>,
) {
    for (action, interaction, previous) in actions.iter() {
        if (Interaction::Hovered, Interaction::Clicked) == (*interaction, *previous.get()) {
            event_writer.send(action.clone())
        }
    }
}

#[cfg(feature = "ui-debug")]
pub fn debug_print_action(mut event_reader: EventReader<UiAction>) {
    for action in event_reader.iter() {
        info!("UiAction fired: {:?}", action)
    }
}
