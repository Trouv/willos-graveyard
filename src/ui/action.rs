//! Plugin for associating UI buttons with logical actions.
//!
//! When a button with a [`UiAction<T>`](UiAction) component is clicked, the plugin will fire an
//! equivalent `UiAction<T>` event.
//! `T` can be any type that you want to use to distinguish between different buttons/actions.

use std::marker::PhantomData;

use crate::previous_component::{
    PreviousComponent, PreviousComponentPlugin, TrackPreviousComponent,
};
use bevy::prelude::*;

/// Set used for detecting UI interactions and firing UiAction events.
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct UiActionSet;

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
        if !app.is_plugin_added::<PreviousComponentPlugin<Interaction>>() {
            app.add_plugins(PreviousComponentPlugin::<Interaction>::default());
        }

        app.add_event::<UiAction<T>>().add_systems(
            Update,
            ui_action::<T>
                .in_set(UiActionSet)
                .after(TrackPreviousComponent),
        );
    }
}

/// Component and event for associating a UI button with an action.
///
/// Insert it on a button to define what action that button performs.
/// Then, when that button is pressed, an event of the same value will be fired.
#[allow(dead_code)]
#[derive(Clone, Eq, PartialEq, Debug, Default, Deref, DerefMut, Component, Event)]
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
        if (Interaction::Hovered, Interaction::Pressed) == (*interaction, *previous.get()) {
            event_writer.write(action.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::SystemState;

    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestAction {
        First,
        Second,
    }

    fn app_setup() -> App {
        let mut app = App::new();

        app.add_plugins(UiActionPlugin::<TestAction>::new());
        app
    }

    fn spawn_two_buttons(app: &mut App) -> (Entity, Entity) {
        (
            app.world_mut()
                .spawn(Button::default())
                .insert(PreviousComponent::<Interaction>::default())
                .insert(UiAction(TestAction::First))
                .id(),
            app.world_mut()
                .spawn(Button::default())
                .insert(PreviousComponent::<Interaction>::default())
                .insert(UiAction(TestAction::Second))
                .id(),
        )
    }

    #[test]
    fn action_fires_on_click() {
        let mut app = app_setup();
        let (first_entity, second_entity) = spawn_two_buttons(&mut app);

        // Simulate the first button click
        *app.world_mut()
            .entity_mut(first_entity)
            .get_mut::<Interaction>()
            .unwrap() = Interaction::Pressed;

        app.update();

        *app.world_mut()
            .entity_mut(first_entity)
            .get_mut::<Interaction>()
            .unwrap() = Interaction::Hovered;

        app.update();

        // Test that the first button click fired an event
        let mut system_state: SystemState<EventReader<UiAction<TestAction>>> =
            SystemState::new(&mut app.world_mut());
        let mut events: EventReader<UiAction<TestAction>> = system_state.get(&app.world());

        assert_eq!(events.len(), 1);
        assert_eq!(*events.read().next().unwrap(), UiAction(TestAction::First));

        // Simulate the second button click, reset the first button
        *app.world_mut()
            .entity_mut(first_entity)
            .get_mut::<Interaction>()
            .unwrap() = Interaction::None;

        *app.world_mut()
            .entity_mut(second_entity)
            .get_mut::<Interaction>()
            .unwrap() = Interaction::Pressed;

        app.update();

        *app.world_mut()
            .entity_mut(second_entity)
            .get_mut::<Interaction>()
            .unwrap() = Interaction::Hovered;

        app.update();

        // Test that the second button click fired an event
        let mut system_state: SystemState<EventReader<UiAction<TestAction>>> =
            SystemState::new(&mut app.world_mut());
        let mut events: EventReader<UiAction<TestAction>> = system_state.get(&app.world());

        assert_eq!(events.len(), 1);
        assert_eq!(*events.read().next().unwrap(), UiAction(TestAction::Second));
    }

    #[test]
    fn action_doesnt_fire_on_drag() {
        let mut app = app_setup();
        let (first_entity, _) = spawn_two_buttons(&mut app);

        // Simulate the button being clicked, and the mouse dragging off before unclicking
        *app.world_mut()
            .entity_mut(first_entity)
            .get_mut::<Interaction>()
            .unwrap() = Interaction::Pressed;

        app.update();

        *app.world_mut()
            .entity_mut(first_entity)
            .get_mut::<Interaction>()
            .unwrap() = Interaction::None;

        app.update();

        // Test that the button drag did not fire an event
        let mut system_state: SystemState<EventReader<UiAction<TestAction>>> =
            SystemState::new(&mut app.world_mut());
        let events: EventReader<UiAction<TestAction>> = system_state.get(&app.world());

        assert_eq!(events.len(), 0);
    }
}
