use crate::{from_component::FromComponentPlugin, gameplay::components::*};
use bevy::prelude::*;
use std::marker::PhantomData;

pub fn sprite_sheet_animation(
    mut query: Query<(Entity, &mut TextureAtlasSprite, &mut SpriteSheetAnimation)>,
    time: Res<Time>,
    mut event_writer: EventWriter<AnimationEvent>,
) {
    for (entity, mut sprite, mut sprite_sheet_animation) in query.iter_mut() {
        sprite_sheet_animation.frame_timer.tick(time.delta());

        if sprite_sheet_animation.frame_timer.just_finished() {
            sprite.index += 1;
            if sprite.index >= sprite_sheet_animation.indices.end {
                sprite.index = sprite_sheet_animation.indices.end - 1;
                event_writer.send(AnimationEvent::Finished(entity));
            }
        }
    }
}

pub fn set_initial_sprite_index(
    mut query: Query<
        (&mut TextureAtlasSprite, &SpriteSheetAnimation),
        Changed<SpriteSheetAnimation>,
    >,
) {
    for (mut sprite, sprite_sheet_animation) in query.iter_mut() {
        let indices = &sprite_sheet_animation.indices;
        if sprite.index < indices.start || sprite.index > indices.end {
            sprite.index = indices.start;
        }
    }
}

pub struct SpriteSheetAnimationPlugin;

impl Plugin for SpriteSheetAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            sprite_sheet_animation.label("animation"), //.before("from_component"),
        )
        .add_system(
            set_initial_sprite_index.label("animation"), //.before("from_component"),
        );
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AnimationEvent {
    Finished(Entity),
}

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
    pub fn new() -> Self {
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
        app.add_plugin(FromComponentPlugin::<F, SpriteSheetAnimation>::new())
            .add_system(animation_finisher::<F>.before("animation"));
    }
}

fn animation_finisher<F>(
    mut query: Query<(&mut F, &mut TextureAtlasSprite, &mut SpriteSheetAnimation)>,
    mut event_reader: EventReader<AnimationEvent>,
) where
    F: Into<SpriteSheetAnimation> + Component + 'static + Send + Sync + Clone + Iterator<Item = F>,
{
    for event in event_reader.iter() {
        match event {
            AnimationEvent::Finished(entity) => {
                if let Ok((mut from, mut sprite, mut sprite_sheet_animation)) =
                    query.get_mut(*entity)
                {
                    *from = from.next().unwrap();
                    *sprite_sheet_animation = from.clone().into();
                    sprite.index = sprite_sheet_animation.indices.start;
                }
            }
        }
    }
}
