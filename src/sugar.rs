use crate::gameplay::Direction;
use crate::{
    animation::SpriteSheetAnimation,
    gameplay::{components::*, *},
    history::HistoryEvent,
    resources::*,
    *,
};
use bevy::{prelude::*, utils::Duration};
use std::{cmp, ops::Range};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Component)]
pub enum PlayerAnimationState {
    Idle,
    Push(Direction),
    Move(Direction),
    Dying,
    None,
}

impl Default for PlayerAnimationState {
    fn default() -> Self {
        PlayerAnimationState::Idle
    }
}

impl Iterator for PlayerAnimationState {
    type Item = Self;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self {
            PlayerAnimationState::Dying | PlayerAnimationState::None => PlayerAnimationState::None,
            PlayerAnimationState::Push(d) => PlayerAnimationState::Move(*d),
            _ => PlayerAnimationState::Idle,
        })
    }
}

impl From<PlayerAnimationState> for SpriteSheetAnimation {
    fn from(state: PlayerAnimationState) -> SpriteSheetAnimation {
        use Direction::*;
        use PlayerAnimationState::*;

        let indices = match state {
            Idle => 0..6,
            Move(Up) => 25..31,
            Move(Down) => 50..56,
            Move(Left) => 75..81,
            Move(Right) => 100..106,
            Push(Up) => 126..128,
            Push(Down) => 151..153,
            Push(Left) => 176..178,
            Push(Right) => 201..203,
            Dying => 225..250,
            None => 7..8,
        };

        let frame_timer = match state {
            _ => Timer::new(Duration::from_millis(150), true),
        };

        SpriteSheetAnimation {
            indices,
            frame_timer,
        }
    }
}

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
}

impl Default for GoalAnimationState {
    fn default() -> Self {
        GoalAnimationState::Idle
    }
}

#[derive(Clone, Default, Debug, Component)]
pub struct GoalGhostAnimation {
    pub column: usize,
    pub frame_timer: Timer,
    pub frames_since_blink: usize,
    pub frames_since_turn: usize,
    pub state: GoalAnimationState,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum DeathHoleState {
    Opening,
    Closed,
}

impl Iterator for DeathHoleState {
    type Item = DeathHoleState;
    fn next(&mut self) -> Option<Self::Item> {
        *self = DeathHoleState::Closed;
        Some(*self)
    }
}

impl From<DeathHoleState> for SpriteSheetAnimation {
    fn from(state: DeathHoleState) -> SpriteSheetAnimation {
        SpriteSheetAnimation {
            indices: match state {
                DeathHoleState::Opening => 0..29,
                DeathHoleState::Closed => 29..30,
            },
            frame_timer: Timer::new(Duration::from_millis(150), true),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum DemonArmsState {
    Grabbing,
    Gone,
}

impl Iterator for DemonArmsState {
    type Item = DemonArmsState;
    fn next(&mut self) -> Option<Self::Item> {
        *self = DemonArmsState::Gone;
        Some(*self)
    }
}

impl From<DemonArmsState> for SpriteSheetAnimation {
    fn from(state: DemonArmsState) -> SpriteSheetAnimation {
        SpriteSheetAnimation {
            indices: match state {
                DemonArmsState::Grabbing => 0..29,
                DemonArmsState::Gone => 29..30,
            },
            frame_timer: Timer::new(Duration::from_millis(150), true),
        }
    }
}

pub fn history_sugar(
    mut history_event_reader: EventReader<HistoryEvent>,
    mut player_query: Query<&mut PlayerAnimationState>,
    audio: Res<Audio>,
    sfx: Res<SoundEffects>,
) {
    for event in history_event_reader.iter() {
        match event {
            HistoryEvent::Rewind | HistoryEvent::Reset => {
                *player_query.single_mut() = PlayerAnimationState::Idle;
                audio.play(sfx.undo.clone_weak());
            }
            _ => (),
        }
    }
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
