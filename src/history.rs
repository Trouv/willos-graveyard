use bevy::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HistoryEvent {
    Record,
    Rewind,
    Reset,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemLabel)]
pub struct FlushHistoryCommands;

#[derive(Clone, PartialEq, Debug, Default, Component, Deref, DerefMut)]
pub struct History<C: Component>(Vec<C>);

pub fn flush_history_commands<C: Component + Clone + core::fmt::Debug>(
    mut history_query: Query<(&mut History<C>, &mut C)>,
    mut history_events: EventReader<HistoryEvent>,
) {
    for event in history_events.iter() {
        match event {
            HistoryEvent::Record => {
                for (mut history, component) in history_query.iter_mut() {
                    history.push(component.clone());
                    dbg!(history);
                }
            }
            HistoryEvent::Rewind => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some(prev_state) = history.pop() {
                        *component = prev_state;
                    }
                    dbg!(history);
                }
            }
            HistoryEvent::Reset => {
                for (mut history, mut component) in history_query.iter_mut() {
                    if let Some(first) = history.get(0) {
                        // Cloning is done before pushing to avoid borrow check issues
                        let first = first.clone();

                        history.push(component.clone());

                        // Updating to a clone of the first item instead of rewinding the entire
                        // list allows us to rewind the act of resetting.
                        *component = first;
                    }
                    dbg!(history);
                }
            }
        }
    }
}
