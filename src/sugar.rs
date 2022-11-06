use crate::{
    gameplay::{components::*, *},
    resources::*,
    *,
};
use bevy::prelude::*;
use std::{cmp, ops::Range};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HandDirection {
    Right,
    Left,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GoalAnimationState {
    Idle,
    Turn { hand: HandDirection, frames: usize },
    Blinking { frames: usize },
    Happy { frame: usize },
    None,
}

impl Default for GoalAnimationState {
    fn default() -> Self {
        GoalAnimationState::Idle
    }
}

#[derive(Clone, Debug, Component)]
pub struct GoalGhostAnimation {
    pub goal_entity: Entity,
    pub frame_timer: Timer,
    pub column: usize,
    pub frames_since_blink: usize,
    pub frames_since_turn: usize,
    pub state: GoalAnimationState,
}

impl GoalGhostAnimation {
    pub fn new(goal_entity: Entity, frame_timer: Timer) -> GoalGhostAnimation {
        GoalGhostAnimation {
            goal_entity,
            frame_timer,
            column: 0,
            frames_since_blink: 0,
            frames_since_turn: 0,
            state: GoalAnimationState::default(),
        }
    }
}

pub fn goal_ghost_event_sugar(
    mut goal_ghost_query: Query<&mut GoalGhostAnimation>,
    mut goal_events: EventReader<GoalEvent>,
) {
    for event in goal_events.iter() {
        for mut animation in goal_ghost_query.iter_mut() {
            match event {
                GoalEvent::Met { goal_entity, .. } => {
                    if *goal_entity == animation.goal_entity {
                        animation.state = GoalAnimationState::Happy { frame: 0 };
                    }
                }
                GoalEvent::UnMet { goal_entity } => {
                    if *goal_entity == animation.goal_entity {
                        animation.state = GoalAnimationState::Idle;
                    }
                }
            }
        }
    }
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
                GoalAnimationState::Happy { frame } => {
                    let index_offset = goal_ghost_settings.num_columns * 4;

                    sprite.index = index_offset + frame;

                    if animation.column >= goal_ghost_settings.num_columns {
                        sprite.index = animation.column;
                    }

                    if frame < goal_ghost_settings.happy_frame_count - 1 {
                        animation.state = GoalAnimationState::Happy { frame: frame + 1 };
                    } else {
                        animation.state = GoalAnimationState::None;
                    }
                }
                GoalAnimationState::None => {
                    sprite.index = goal_ghost_settings.none_frame_index;
                }
            }

            animation.column += 1;
            animation.column %= goal_ghost_settings.idle_frame_count;
        }
    }
}

fn range_chance(range: &Range<usize>, current: usize) -> f32 {
    ((current as f32 - range.start as f32) / (range.end as f32 - range.start as f32)).clamp(0., 1.)
}

pub fn animate_grass_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut WindTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let mut rng = rand::thread_rng();
            let chance = rng.gen::<f32>();
            if chance <= 0.2 {
                sprite.index = cmp::min(sprite.index + 1, texture_atlas.len() - 1);
            } else if chance > 0.2 && chance <= 0.6 {
                sprite.index = cmp::max(sprite.index as i32 - 1, 0) as usize;
            }
        }
    }
}

pub fn make_ui_visible(mut ui_query: Query<&mut Visibility, With<Node>>) {
    for mut visibility in ui_query.iter_mut() {
        visibility.is_visible = true;
    }
}
