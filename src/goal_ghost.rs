use crate::{
    gameplay::{components::*, GoalEvent,},utils::range_chance
};
use std::{ops::Range, time::Duration};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use rand::Rng;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum HandDirection {
    Right,
    Left,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    #[grid_coords]
    pub grid_coords: GridCoords,
    pub goal: Goal,
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
}

pub fn spawn_goal_ghosts(
    mut commands: Commands,
    goals: Query<Entity, Added<Goal>>,
    mut goal_ghost_settings: ResMut<GoalGhostSettings>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for goal_entity in goals.iter() {
        let atlas_handle = match &goal_ghost_settings.atlas {
            Some(atlas) => atlas.clone(),
            None => {
                let image_handle = asset_server.load("textures/animations/goal_ghost-Sheet.png");
                let texture_atlas = TextureAtlas::from_grid(
                    image_handle,
                    Vec2::splat(32.),
                    goal_ghost_settings.num_columns,
                    goal_ghost_settings.num_rows,
                );
                let atlas_handle = texture_atlases.add(texture_atlas);

                goal_ghost_settings.atlas = Some(atlas_handle.clone());
                atlas_handle.clone()
            }
        };

        let ghost_entity = commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: atlas_handle,
                transform: Transform::from_xyz(0., 1., 2.5),
                ..default()
            })
            .insert(GoalGhostAnimation::new(
                goal_entity,
                Timer::new(goal_ghost_settings.frame_duration, true),
            ))
            .id();

        commands.entity(goal_entity).add_child(ghost_entity);
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct GoalGhostSettings {
    pub no_turn_length: Range<usize>,
    pub turn_length: Range<usize>,
    pub no_blink_length: Range<usize>,
    pub blink_length: Range<usize>,
    pub frame_duration: Duration,
    pub idle_frame_count: usize,
    pub happy_frame_count: usize,
    pub none_frame_index: usize,
    pub num_columns: usize,
    pub num_rows: usize,
    pub atlas: Option<Handle<TextureAtlas>>,
    pub punctuation_timer: Range<usize>,
}

impl GoalGhostSettings {
    pub const NORMAL: GoalGhostSettings = GoalGhostSettings {
        no_turn_length: 32..64,
        turn_length: 12..20,
        no_blink_length: 50..100,
        blink_length: 0..1,
        frame_duration: Duration::from_millis(150),
        idle_frame_count: 8,
        happy_frame_count: 10,
        none_frame_index: 8,
        num_columns: 10,
        num_rows: 5,
        atlas: None,
        punctuation_timer: 40..100,
    };
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
    pub frames_since_punctuation: usize,
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
            frames_since_punctuation: 0,
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