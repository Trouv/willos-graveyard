//! Plugin that tracks history, rewinds, and resets gamestate for arbitrary components.
use bevy::prelude::*;
use std::any::Any;
use std::marker::PhantomData;

/// Plugin that tracks history, rewinds, and resets gamestate for arbitrary components.
pub struct HistoryPlugin<C: Component + Clone, S: States> {
    state: S,
    phantom: PhantomData<C>,
}

impl<C: Component + Clone, S: States> HistoryPlugin<C, S> {
    /// Constructor for the plugin.
    ///
    /// Allows the user to specify a particular iyes_loopless state to run the plugin in.
    pub fn run_in_state(state: S) -> Self {
        HistoryPlugin {
            state,
            phantom: PhantomData::<C>,
        }
    }
}

impl<C: Component + Clone, S: States> Plugin for HistoryPlugin<C, S> {
    fn build(&self, app: &mut App) {
        app.add_event::<HistoryCommands>().add_system(
            flush_history_commands::<C>
                .run_if(in_state(self.state.clone()))
                .in_set(FlushHistoryCommands),
        );
    }
}

/// Event that can be fired by the user to command the plugin to perform various history tasks.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HistoryCommands {
    /// Record the current state of all tracked components to their histories.
    Record,
    /// Update the current state of all tracked components with the previous state and remove it
    /// from the history.
    Rewind,
    /// Update the current state of all tracked components to the first state in the history.
    ///
    /// Note: This also records the current state to the history before updating it.
    /// This allows the act of resetting the history to be rewound via [HistoryCommands::Rewind].
    Reset,
}

/// System label for the system that handles history commands.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemSet)]
pub struct FlushHistoryCommands;

/// Component that stores the history of another component generically.
#[derive(Clone, PartialEq, Eq, Debug, Default, Component, Deref, DerefMut)]
pub struct History<C: Component + Clone>(Vec<C>);

fn flush_history_commands<C: Component + Clone>(
    mut history_query: Query<(&mut History<C>, &mut C)>,
    mut history_commands: EventReader<HistoryCommands>,
) {
    for command in history_commands.iter() {
        match command {
            HistoryCommands::Record => {
                for (mut history, component) in history_query.iter_mut() {
                    history.push(component.clone());
                }
            }
            HistoryCommands::Rewind => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some(prev_state) = history.pop() {
                        *component = prev_state;
                    }
                }
            }
            HistoryCommands::Reset => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some(first) = history.get(0) {
                        // Cloning is done before pushing to avoid borrow check issues
                        let first = first.clone();

                        history.push(component.clone());

                        // Updating to a clone of the first item instead of rewinding the entire
                        // list allows us to rewind the act of resetting.
                        *component = first;
                    }
                }
            }
        }
    }
}
