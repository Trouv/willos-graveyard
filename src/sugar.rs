use crate::gameplay::Direction;
use crate::{
    animation::SpriteSheetAnimation,
    gameplay::{components::*, xy_translation, *},
    history::HistoryCommands,
    resources::*,
    *,
};
use bevy::{prelude::*, utils::Duration};
use bevy_easings::*;
use std::{cmp, ops::Range};

pub fn ease_movement(
    mut commands: Commands,
    mut grid_coords_query: Query<
        (
            Entity,
            &GridCoords,
            &Transform,
            Option<&PlayerAnimationState>,
        ),
        (Changed<GridCoords>, Without<MoveTable>),
    >,
) {
    for (entity, &grid_coords, transform, player_state) in grid_coords_query.iter_mut() {
        let mut xy = xy_translation(grid_coords.into());

        if let Some(PlayerAnimationState::Push(direction)) = player_state {
            xy += IVec2::from(*direction).as_vec2() * 5.;
        }

        commands.entity(entity).insert(transform.ease_to(
            Transform::from_xyz(xy.x, xy.y, transform.translation.z),
            EaseFunction::CubicOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(110),
            },
        ));
    }
}

pub fn reset_player_easing(
    mut commands: Commands,
    player_query: Query<
        (Entity, &GridCoords, &Transform, &PlayerAnimationState),
        Changed<PlayerAnimationState>,
    >,
) {
    if let Ok((entity, &grid_coords, transform, player_animation_state)) = player_query.get_single()
    {
        match player_animation_state {
            PlayerAnimationState::Push(_) => (),
            _ => {
                let xy = xy_translation(grid_coords.into());
                commands.entity(entity).insert(transform.ease_to(
                    Transform::from_xyz(xy.x, xy.y, transform.translation.z),
                    EaseFunction::CubicOut,
                    EasingType::Once {
                        duration: std::time::Duration::from_millis(110),
                    },
                ));
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Component)]
pub enum PlayerAnimationState {
    Idle(Direction),
    Push(Direction),
    Dying,
    None,
}

impl Default for PlayerAnimationState {
    fn default() -> Self {
        PlayerAnimationState::Idle(Direction::Down)
    }
}

impl Iterator for PlayerAnimationState {
    type Item = Self;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self {
            PlayerAnimationState::Dying | PlayerAnimationState::None => PlayerAnimationState::None,
            PlayerAnimationState::Push(d) => PlayerAnimationState::Idle(*d),
            _ => PlayerAnimationState::Idle(Direction::Down),
        })
    }
}

impl From<PlayerAnimationState> for SpriteSheetAnimation {
    fn from(state: PlayerAnimationState) -> SpriteSheetAnimation {
        use Direction::*;
        use PlayerAnimationState::*;

        let indices = match state {
            Push(Up) => 1..2,
            Push(Down) => 26..27,
            Push(Left) => 51..52,
            Push(Right) => 76..77,
            Idle(Up) => 100..107,
            Idle(Down) => 125..132,
            Idle(Left) => 150..157,
            Idle(Right) => 175..182,
            Dying => 200..225,
            None => 7..8,
        };

        let frame_timer = match state {
            //Push(_) => Timer::new(Duration::from_millis(75), true),
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
    mut history_commands: EventReader<HistoryCommands>,
    mut player_query: Query<&mut PlayerAnimationState>,
    audio: Res<Audio>,
    sfx: Res<SoundEffects>,
) {
    for command in history_commands.iter() {
        match command {
            HistoryCommands::Rewind | HistoryCommands::Reset => {
                *player_query.single_mut() = PlayerAnimationState::Idle(Direction::Down);
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
    mut history_commands: EventReader<HistoryCommands>,
    death_hole_query: Query<Entity, With<DeathHoleState>>,
    demon_arms_query: Query<Entity, With<DemonArmsState>>,
) {
    for command in history_commands.iter() {
        match command {
            HistoryCommands::Rewind | HistoryCommands::Reset => {
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
