use crate::{
    animation::SpriteSheetAnimation,
    gameplay::{
        components::{MoveTable, MovementTimer},
        Direction,
    },
    gameplay::{xy_translation, *},
    history::HistoryCommands,
    resources::{RewindSettings, RewindTimer},
    *,
};
use bevy::{prelude::*, utils::Duration};
use bevy_easings::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct PlayerMovementEvent {
    pub direction: Direction,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum PlayerState {
    Waiting,
    Dead,
    RankMove(KeyCode),
    FileMove(KeyCode),
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState::Waiting
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
            Push(Down) => 11..12,
            Push(Left) => 21..22,
            Push(Right) => 31..32,
            Idle(Up) => 40..47,
            Idle(Down) => 50..57,
            Idle(Left) => 60..67,
            Idle(Right) => 70..77,
            Dying => 80..105,
            None => 3..4,
        };

        let frame_timer = Timer::new(Duration::from_millis(150), true);

        let repeat = matches!(state, Idle(Down));

        SpriteSheetAnimation {
            indices,
            frame_timer,
            repeat,
        }
    }
}

pub fn history_sugar(
    mut history_commands: EventReader<HistoryCommands>,
    mut player_query: Query<&mut PlayerAnimationState>,
    audio: Res<Audio>,
    sfx: Res<AssetHolder>,
) {
    for command in history_commands.iter() {
        match command {
            HistoryCommands::Rewind | HistoryCommands::Reset => {
                *player_query.single_mut() = PlayerAnimationState::Idle(Direction::Down);
                audio.play(sfx.undo_sound.clone_weak());
            }
            _ => (),
        }
    }
}

pub fn play_death_animations(
    mut player_query: Query<&mut PlayerAnimationState>,
    mut death_event_reader: EventReader<DeathEvent>,
) {
    for DeathEvent { player_entity } in death_event_reader.iter() {
        if let Ok(mut player_animation_state) = player_query.get_mut(*player_entity) {
            *player_animation_state = PlayerAnimationState::Dying;
        }
    }
}

pub fn move_player_by_table(
    table_query: Query<&MoveTable>,
    mut player_query: Query<(&mut MovementTimer, &mut PlayerState)>,
    mut movement_writer: EventWriter<PlayerMovementEvent>,
    time: Res<Time>,
) {
    for table in table_query.iter() {
        if let Ok((mut timer, mut player)) = player_query.get_single_mut() {
            timer.0.tick(time.delta());

            if timer.0.finished() {
                match *player {
                    PlayerState::RankMove(key) => {
                        for (i, rank) in table.table.iter().enumerate() {
                            if rank.contains(&Some(key)) {
                                movement_writer.send(PlayerMovementEvent {
                                    direction: DIRECTION_ORDER[i],
                                });
                            }
                        }
                        *player = PlayerState::FileMove(key);
                        timer.0.reset();
                    }
                    PlayerState::FileMove(key) => {
                        for rank in table.table.iter() {
                            for (i, cell) in rank.iter().enumerate() {
                                if *cell == Some(key) {
                                    movement_writer.send(PlayerMovementEvent {
                                        direction: DIRECTION_ORDER[i],
                                    });
                                }
                            }
                        }
                        *player = PlayerState::Waiting;
                        timer.0.reset();
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn player_state_input(
    mut player_query: Query<&mut PlayerState>,
    input: Res<Input<KeyCode>>,
    mut history_commands: EventWriter<HistoryCommands>,
    mut rewind_settings: ResMut<RewindSettings>,
    time: Res<Time>,
) {
    for mut player in player_query.iter_mut() {
        if *player == PlayerState::Waiting {
            if input.just_pressed(KeyCode::W) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::W)
            } else if input.just_pressed(KeyCode::A) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::A)
            } else if input.just_pressed(KeyCode::S) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::S)
            } else if input.just_pressed(KeyCode::D) {
                history_commands.send(HistoryCommands::Record);
                *player = PlayerState::RankMove(KeyCode::D)
            }
        }

        if *player == PlayerState::Waiting || *player == PlayerState::Dead {
            if input.just_pressed(KeyCode::Z) {
                history_commands.send(HistoryCommands::Rewind);
                *player = PlayerState::Waiting;
                rewind_settings.hold_timer =
                    Some(RewindTimer::new(rewind_settings.hold_range_millis.end));
            } else if input.pressed(KeyCode::Z) {
                let range = rewind_settings.hold_range_millis.clone();
                let acceleration = rewind_settings.hold_acceleration;

                if let Some(RewindTimer { velocity, timer }) = &mut rewind_settings.hold_timer {
                    *velocity = (*velocity - (acceleration * time.delta_seconds()))
                        .clamp(range.start as f32, range.end as f32);

                    timer.tick(time.delta());

                    if timer.just_finished() {
                        history_commands.send(HistoryCommands::Rewind);
                        *player = PlayerState::Waiting;

                        timer.set_duration(Duration::from_millis(*velocity as u64));
                    }
                }
            } else if input.just_pressed(KeyCode::R) {
                history_commands.send(HistoryCommands::Reset);
                *player = PlayerState::Waiting;
            }
        }
    }
}
