use crate::previous_component::PreviousComponent;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub enum UiAction {
    Debug(&'static str),
    RestartLevel,
    NextLevel,
    GoToLevel(LevelSelection),
}

pub fn ui_action(
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
        match action {
            UiAction::Debug(s) => info!("{}", s),
            _ => (),
        }
    }
}
