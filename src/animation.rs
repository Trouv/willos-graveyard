use crate::{
    from_component::FromComponentPlugin, gameplay::components::*, gameplay::*, resources::*,
};
use bevy::prelude::*;
use rand::prelude::*;
use std::marker::PhantomData;
use std::ops::Range;

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
                } else {
                    warn!("Unable to find from component for finished animation");
                }
            }
        }
    }
}

fn range_chance(range: &Range<usize>, current: usize) -> f32 {
    ((current as f32 - range.start as f32) / (range.end as f32 - range.start as f32)).clamp(0., 1.)
}

pub fn goal_ghost_animation(
    mut goal_ghost_query: Query<(&mut GoalGhostAnimation, &mut TextureAtlasSprite)>,
    goal_ghost_settings: Res<GoalGhostSettings>,
    time: Res<Time>,
) {
    for (mut animation, mut sprite) in goal_ghost_query.iter_mut() {
        animation.frame_timer.tick(time.delta());

        if animation.frame_timer.finished() {
            let mut rng = rand::thread_rng();

            match animation.state {
                GoalAnimationState::Idle => {
                    sprite.index = animation.column;

                    let chance_to_turn = range_chance(
                        &goal_ghost_settings.no_turn_length,
                        animation.frames_since_turn,
                    );

                    let chance_to_blink = range_chance(
                        &goal_ghost_settings.no_blink_length,
                        animation.frames_since_blink,
                    );

                    let r: f32 = rng.gen();

                    if r < chance_to_turn {
                        let hand = if rng.gen::<f32>() < 0.5 {
                            HandDirection::Right
                        } else {
                            HandDirection::Left
                        };

                        animation.state = GoalAnimationState::Turn { hand, frames: 0 };
                    } else if r < chance_to_blink {
                        animation.state = GoalAnimationState::Blinking { frames: 0 };
                    }

                    animation.frames_since_turn += 1;
                    animation.frames_since_blink += 1;
                }
                GoalAnimationState::Turn { hand, frames } => {
                    let index_offset = if hand == HandDirection::Right {
                        goal_ghost_settings.num_columns
                    } else {
                        goal_ghost_settings.num_columns * 2
                    };

                    sprite.index = index_offset + animation.column;

                    let chance_animation_ends =
                        range_chance(&goal_ghost_settings.turn_length, frames);

                    if rng.gen::<f32>() < chance_animation_ends {
                        animation.state = GoalAnimationState::Idle;
                    } else {
                        animation.state = GoalAnimationState::Turn {
                            hand,
                            frames: frames + 1,
                        };
                    }

                    animation.frames_since_turn = 0;
                }
                GoalAnimationState::Blinking { frames } => {
                    let index_offset = goal_ghost_settings.num_columns * 3;

                    sprite.index = index_offset + animation.column;

                    let chance_animation_ends =
                        range_chance(&goal_ghost_settings.blink_length, frames);

                    if rng.gen::<f32>() < chance_animation_ends {
                        animation.state = GoalAnimationState::Idle;
                    } else {
                        animation.state = GoalAnimationState::Blinking { frames: frames + 1 };
                    }
                    animation.frames_since_blink = 0;
                }
            }

            animation.column += 1;
            animation.column %= goal_ghost_settings.num_columns;
        }
    }
}

pub fn load_death_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let death_hole_texture = asset_server.load("textures/animations/death-Sheet.png");
    let demon_arms_texture = asset_server.load("textures/animations/demon-Sheet.png");

    let death_hole_texture_atlas =
        TextureAtlas::from_grid(death_hole_texture, Vec2::splat(32.), 30, 1);
    let demon_arms_texture_atlas =
        TextureAtlas::from_grid(demon_arms_texture, Vec2::splat(32.), 30, 1);

    commands.insert_resource(DeathAnimationTextureAtlases {
        death_hole_handle: texture_atlases.add(death_hole_texture_atlas),
        demon_arms_handle: texture_atlases.add(demon_arms_texture_atlas),
    });
}

pub fn play_death_animations(
    mut commands: Commands,
    mut player_query: Query<&mut PlayerAnimationState>,
    mut death_event_reader: EventReader<DeathEvent>,
    death_animation_texture_atlases: Res<DeathAnimationTextureAtlases>,
) {
    for DeathEvent {
        player_entity,
        exorcism_entity,
    } in death_event_reader.iter()
    {
        if let Ok(mut player_animation_state) = player_query.get_mut(*player_entity) {
            *player_animation_state = PlayerAnimationState::Dying;
        }

        commands
            .entity(*exorcism_entity)
            .with_children(|child_commands| {
                child_commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: death_animation_texture_atlases.death_hole_handle.clone(),
                        transform: Transform::from_xyz(0., 0., 0.5),
                        ..default()
                    })
                    .insert(DeathHoleState::Opening);

                child_commands
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: death_animation_texture_atlases.demon_arms_handle.clone(),
                        transform: Transform::from_xyz(0., 0., 1.5),
                        ..default()
                    })
                    .insert(DemonArmsState::Grabbing);
            });
    }
}

pub fn despawn_death_animations(
    mut commands: Commands,
    mut history_event_reader: EventReader<HistoryEvent>,
    death_hole_query: Query<Entity, With<DeathHoleState>>,
    demon_arms_query: Query<Entity, With<DemonArmsState>>,
) {
    for event in history_event_reader.iter() {
        match event {
            HistoryEvent::Rewind | HistoryEvent::Reset => {
                dbg!(event);
                for entity in death_hole_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                for entity in demon_arms_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            }
            _ => (),
        }
    }
}
