//! Plugin providing functionality for basic sprite sheet animations.
use crate::from_component::{FromComponentPlugin, FromComponentSet};
use bevy::{ecs::component::Mutable, prelude::*};
use std::{marker::PhantomData, ops::Range};

/// Set used by animation systems.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, SystemSet)]
pub struct AnimationSet;

/// Plugin providing functionality for basic sprite sheet animations.
pub struct SpriteSheetAnimationPlugin;

impl Plugin for SpriteSheetAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationEvent>().add_systems(
            Update,
            (
                sprite_sheet_animation
                    .in_set(AnimationSet)
                    .after(FromComponentSet),
                set_initial_sprite_index
                    .in_set(AnimationSet)
                    .after(FromComponentSet),
            ),
        );
    }
}

/// Event that fires at certain points during an animation.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Event)]
pub enum AnimationEvent {
    /// Event that fires when an animation finishes, storing the animated entity.
    Finished(Entity),
}

/// Component for giving a sprite sheet bundle a basic animation, with some settings for its
/// behaviour.
#[derive(Clone, Debug, Default, Component)]
pub struct SpriteSheetAnimation {
    /// The range of indices of the texture atlas that provide the frames of the animation.
    pub indices: Range<usize>,
    /// Timer that defines the duration of a frame in the animation and tracks its progress.
    ///
    /// Currently, the timer mode must be repeating.
    // TODO: provide a constructor privatizing this field so it's always repeating.
    pub frame_timer: Timer,
    /// Whether the animation should loop or not.
    ///
    /// Note: Animations that loop never fire [AnimationEvent::Finished], and can never
    /// automatically transition to another animation in an animation graph.
    pub repeat: bool,
}

fn sprite_sheet_animation(
    mut query: Query<(Entity, &mut Sprite, &mut SpriteSheetAnimation)>,
    time: Res<Time>,
    mut event_writer: EventWriter<AnimationEvent>,
) {
    for (entity, mut sprite, mut sprite_sheet_animation) in query.iter_mut() {
        sprite_sheet_animation.frame_timer.tick(time.delta());
        let Some(sprite) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        if sprite_sheet_animation.frame_timer.just_finished() {
            sprite.index += 1;
            if sprite.index >= sprite_sheet_animation.indices.end {
                // Animation finished
                if sprite_sheet_animation.repeat {
                    sprite.index = sprite_sheet_animation.indices.start;
                } else {
                    sprite.index = sprite_sheet_animation.indices.end - 1;
                    event_writer.send(AnimationEvent::Finished(entity));
                }
            }
        }
    }
}

fn set_initial_sprite_index(
    mut query: Query<(&mut Sprite, &SpriteSheetAnimation), Changed<SpriteSheetAnimation>>,
) {
    for (mut sprite, sprite_sheet_animation) in query.iter_mut() {
        let Some(sprite) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        let indices = &sprite_sheet_animation.indices;
        if sprite.index < indices.start || sprite.index > indices.end {
            sprite.index = indices.start;
        }
    }
}

/// Plugin providing functionality for animation graphs through `From` and `Iterator`
/// implementations.
///
/// To use this plugin, follow these steps:
/// 1. Create an enum component of animation states.
/// 2. Provide an implementation of `From` from your enum component to [SpriteSheetAnimation].
///    This `From` implementation defines what specific animation settings are associated with
///    variants of your animation state.
/// 3. Provide an implementation of `Iterator` for your enum component with itself as the item.
///    This `Iterator` implementation defines which states lead into other states, like an animation
///    graph.
///    Simply point an animation state to itself when it is complete.
pub struct FromComponentAnimator<F>
where
    F: Into<SpriteSheetAnimation> + Component + 'static + Send + Sync + Clone + Iterator<Item = F>,
{
    from_type: PhantomData<F>,
}

impl<F> FromComponentAnimator<F>
where
    F: Into<SpriteSheetAnimation> + Component + 'static + Send + Sync + Clone + Iterator<Item = F>,
{
    /// Basic constructor for [FromComponentAnimator].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<F> Default for FromComponentAnimator<F>
where
    F: Into<SpriteSheetAnimation> + Component + 'static + Send + Sync + Clone + Iterator<Item = F>,
{
    fn default() -> Self {
        FromComponentAnimator {
            from_type: PhantomData,
        }
    }
}

impl<F> Plugin for FromComponentAnimator<F>
where
    F: Into<SpriteSheetAnimation> + Component + 'static + Send + Sync + Clone + Iterator<Item = F>,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(FromComponentPlugin::<F, SpriteSheetAnimation>::new())
            .add_systems(Update, animation_finisher::<F>.before(AnimationSet));
    }
}

fn animation_finisher<F>(
    mut query: Query<(&mut F, &mut Sprite, &mut SpriteSheetAnimation)>,
    mut event_reader: EventReader<AnimationEvent>,
) where
    F: Into<SpriteSheetAnimation>
        + Component<Mutability = Mutable>
        + 'static
        + Send
        + Sync
        + Clone
        + Iterator<Item = F>,
{
    for event in event_reader.read() {
        match event {
            AnimationEvent::Finished(entity) => {
                if let Ok((mut from, mut sprite, mut sprite_sheet_animation)) =
                    query.get_mut(*entity)
                {
                    let Some(sprite) = sprite.texture_atlas.as_mut() else {
                        continue;
                    };
                    *from = from.next().unwrap();
                    *sprite_sheet_animation = from.clone().into();
                    sprite.index = sprite_sheet_animation.indices.start;
                }
            }
        }
    }
}
