use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemLabel)]
pub struct FromComponentLabel;

pub struct FromComponentPlugin<F, I>
where
    F: Into<I> + Component + 'static + Send + Sync + Clone,
    I: Component + 'static + Send + Sync,
{
    from_type: PhantomData<F>,
    into_type: PhantomData<I>,
}

impl<F, I> FromComponentPlugin<F, I>
where
    F: Into<I> + Component + 'static + Send + Sync + Clone,
    I: Component + 'static + Send + Sync,
{
    pub fn new() -> Self {
        FromComponentPlugin {
            from_type: PhantomData,
            into_type: PhantomData,
        }
    }
}

impl<F, I> Plugin for FromComponentPlugin<F, I>
where
    F: Into<I> + Component + 'static + Send + Sync + Clone,
    I: Component + 'static + Send + Sync,
{
    fn build(&self, app: &mut App) {
        app.add_system(from_changed_component::<F, I>.label(FromComponentLabel))
            .add_system(from_added_component::<F, I>.label(FromComponentLabel));
    }
}

fn from_changed_component<F, I>(mut query: Query<(&F, &mut I), Changed<F>>)
where
    F: Into<I> + Component + 'static + Send + Sync + Clone,
    I: Component + 'static + Send + Sync,
{
    for (from_component, mut into_component) in query.iter_mut() {
        *into_component = from_component.clone().into();
    }
}

fn from_added_component<F, I>(mut commands: Commands, query: Query<(Entity, &F), Added<F>>)
where
    F: Into<I> + Component + 'static + Send + Sync + Clone,
    I: Component + 'static + Send + Sync,
{
    for (entity, from_component) in query.iter() {
        let into: I = from_component.clone().into();
        commands.entity(entity).insert(into);
    }
}
