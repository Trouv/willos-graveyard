use bevy::prelude::*;
use iyes_loopless::prelude::*;
use std::any::Any;
use std::marker::PhantomData;

pub struct HistoryPlugin<C: Component + Clone, S> {
    state: S,
    phantom: PhantomData<C>,
}

impl<C: Component + Clone, S> HistoryPlugin<C, S> {
    pub fn run_in_state(state: S) -> Self {
        HistoryPlugin {
            state,
            phantom: PhantomData::<C>,
        }
    }
}

impl<C: Component + Clone, S> Plugin for HistoryPlugin<C, S>
where
    S: Any + Send + Sync + Clone + std::fmt::Debug + std::hash::Hash + Eq,
{
    fn build(&self, app: &mut App) {
        app.add_event::<HistoryCommands>().add_system(
            flush_history_commands::<C>
                .run_in_state(self.state.clone())
                .label(FlushHistoryCommands),
        );
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HistoryCommands {
    Record,
    Rewind,
    Reset,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemLabel)]
pub struct FlushHistoryCommands;

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
