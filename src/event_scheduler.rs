//! Plugin that provides functionality for scheduling events to fire in the future.
use bevy::prelude::*;
use std::{collections::VecDeque, marker::PhantomData, time::Duration};

/// Plugin that provides functionality for scheduling events to fire in the future.
///
/// Supply the event type with the generic parameter.
/// This plugin needs to be added once per event you plan to do scheduling for.
pub struct EventSchedulerPlugin<E> {
    data: PhantomData<E>,
}

impl<E> Plugin for EventSchedulerPlugin<E>
where
    E: 'static + Send + Sync,
{
    fn build(&self, app: &mut App) {
        app.add_event::<E>()
            .add_system(fire_scheduled_events::<E>)
            .init_resource::<EventScheduler<E>>();
    }
}

impl<E> EventSchedulerPlugin<E>
where
    E: 'static + Send + Sync,
{
    /// Construct a new [EventSchedulerPlugin].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<E> Default for EventSchedulerPlugin<E>
where
    E: 'static + Send + Sync,
{
    fn default() -> Self {
        EventSchedulerPlugin::<E> { data: PhantomData }
    }
}

/// Resource providing the API for scheduling events to fire in the future.
#[derive(Clone, Debug, Resource)]
pub struct EventScheduler<E>
where
    E: 'static + Send + Sync,
{
    events: VecDeque<(E, Timer)>,
}

/// Custom default impl because deriving Default makes it only default when E is default.
impl<E> Default for EventScheduler<E>
where
    E: 'static + Send + Sync,
{
    fn default() -> Self {
        EventScheduler::<E> {
            events: VecDeque::new(),
        }
    }
}

impl<E> EventScheduler<E>
where
    E: 'static + Send + Sync,
{
    /// Schedule an event to fire in the future.
    pub fn schedule(&mut self, event: E, duration: Duration) {
        self.events
            .push_back((event, Timer::new(duration, TimerMode::Once)));
    }
}

fn fire_scheduled_events<E>(
    time: Res<Time>,
    mut event_scheduler: ResMut<EventScheduler<E>>,
    mut writer: EventWriter<E>,
) where
    E: 'static + Send + Sync,
{
    event_scheduler.events = event_scheduler
        .events
        .drain(..)
        .filter_map(|(event, mut timer)| {
            timer.tick(time.delta());

            if timer.finished() {
                writer.send(event);
                None
            } else {
                Some((event, timer))
            }
        })
        .collect();
}
