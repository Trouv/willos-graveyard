//! Plugin that maintains `Into` components from entities' corresponding `From` component.
use bevy::prelude::*;
use std::marker::PhantomData;

/// Set used by systems in the [FromComponentPlugin].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemSet)]
pub struct FromComponentSet;

/// Plugin that maintains `Into` components from entities' corresponding `From` component.
///
/// Generic paramaters `F` and `I` define the `From` and `Into` components the plugin should apply
/// to respectively.
/// The `Into` component should implement `From<..>` the `From` component.
/// Then, when an entity gains the `From` component, this plugin will also give it the
/// corresponding `Into` component.
/// The plugin will also update the `Into` component whenever the `From` component changes.
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
    /// Construct a new [FromComponentPlugin].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<F, I> Default for FromComponentPlugin<F, I>
where
    F: Into<I> + Component + 'static + Send + Sync + Clone,
    I: Component + 'static + Send + Sync,
{
    fn default() -> Self {
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
        app.add_system(from_changed_component::<F, I>.label(FromComponentSet))
            .add_system(from_added_component::<F, I>.label(FromComponentSet));
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
