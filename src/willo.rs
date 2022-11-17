//! Plugin, components and events providing functionality for Willo, the player character.
use crate::{
    animation::{FromComponentAnimator, SpriteSheetAnimation},
    exorcism::ExorcismEvent,
    history::{FlushHistoryCommands, History, HistoryCommands, HistoryPlugin},
    movement_table::Direction,
    sokoban::RigidBody,
    AssetHolder, GameState, SystemLabels, UNIT_LENGTH,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation_centered};
use iyes_loopless::prelude::*;
use std::{ops::Range, time::Duration};

/// Plugin providing functionality for Willo, the player character.
pub struct WilloPlugin;

impl Plugin for WilloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FromComponentAnimator::<WilloAnimationState>::new())
            .add_plugin(HistoryPlugin::<GridCoords, _>::run_in_state(
                GameState::Gameplay,
            ))
            .init_resource::<RewindSettings>()
            .add_event::<WilloMovementEvent>()
            .add_system(
                willo_input
                    .run_in_state(GameState::Gameplay)
                    .label(SystemLabels::Input)
                    .before(FlushHistoryCommands),
            )
            // Systems with potential easing end/beginning collisions cannot be in CoreStage::Update
            // see https://github.com/vleue/bevy_easings/issues/23
            .add_system_to_stage(
                CoreStage::PostUpdate,
                reset_willo_easing
                    .run_not_in_state(GameState::AssetLoading)
                    .before("ease_movement"),
            )
            .add_system(play_death_animations.run_not_in_state(GameState::AssetLoading))
            .add_system(history_sugar.run_not_in_state(GameState::AssetLoading))
            .register_ldtk_entity::<WilloBundle>("Willo");
    }
}

/// Event that fires whenever Willo moves.
///
/// Only fires once per direction - so it fires twice during most moves.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct WilloMovementEvent {
    pub direction: Direction,
}

/// Component that marks Willo and keeps track of their state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum WilloState {
    Waiting,
    Dead,
    RankMove(KeyCode),
    FileMove(KeyCode),
}

impl Default for WilloState {
    fn default() -> WilloState {
        WilloState::Waiting
    }
}

/// Component enumerating the possible states of Willo's animation.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Component)]
pub enum WilloAnimationState {
    Idle(Direction),
    Push(Direction),
    Dying,
    None,
}

impl Default for WilloAnimationState {
    fn default() -> Self {
        WilloAnimationState::Idle(Direction::Down)
    }
}

impl Iterator for WilloAnimationState {
    type Item = Self;
    fn next(&mut self) -> Option<Self::Item> {
        Some(match self {
            WilloAnimationState::Dying | WilloAnimationState::None => WilloAnimationState::None,
            WilloAnimationState::Push(d) => WilloAnimationState::Idle(*d),
            _ => WilloAnimationState::Idle(Direction::Down),
        })
    }
}

impl From<WilloAnimationState> for SpriteSheetAnimation {
    fn from(state: WilloAnimationState) -> SpriteSheetAnimation {
        use Direction::*;
        use WilloAnimationState::*;

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

const MOVEMENT_SECONDS: f32 = 0.14;

/// Component that provides the timer used to space out the movements Willo performs.
#[derive(Clone, Debug, Component)]
pub struct MovementTimer(pub Timer);

impl Default for MovementTimer {
    fn default() -> MovementTimer {
        MovementTimer(Timer::from_seconds(MOVEMENT_SECONDS, false))
    }
}

/// Part of the [RewindSettings] resource.
///
/// Provides space between rewinds and tracking rewind velocity for acceleration.
#[derive(Clone, Debug, Default)]
struct RewindTimer {
    velocity: f32,
    timer: Timer,
}

impl RewindTimer {
    fn new(millis: u64) -> RewindTimer {
        RewindTimer {
            velocity: millis as f32,
            timer: Timer::new(Duration::from_millis(millis), true),
        }
    }
}

/// Resource defining the behavior of the rewind feature and storing its state for acceleration.
#[derive(Clone, Debug)]
struct RewindSettings {
    hold_range_millis: Range<u64>,
    hold_acceleration: f32,
    hold_timer: Option<RewindTimer>,
}

impl Default for RewindSettings {
    fn default() -> Self {
        RewindSettings {
            hold_range_millis: 50..200,
            hold_acceleration: 50.,
            hold_timer: None,
        }
    }
}

#[derive(Clone, Bundle, LdtkEntity)]
struct WilloBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    history: History<GridCoords>,
    #[from_entity_instance]
    rigid_body: RigidBody,
    willo_state: WilloState,
    movement_timer: MovementTimer,
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    willo_animation_state: WilloAnimationState,
}

fn reset_willo_easing(
    mut commands: Commands,
    willo_query: Query<
        (Entity, &GridCoords, &Transform, &WilloAnimationState),
        Changed<WilloAnimationState>,
    >,
) {
    if let Ok((entity, &grid_coords, transform, animation_state)) = willo_query.get_single() {
        match animation_state {
            WilloAnimationState::Push(_) => (),
            _ => {
                let xy =
                    grid_coords_to_translation_centered(grid_coords, IVec2::splat(UNIT_LENGTH));
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

fn history_sugar(
    mut history_commands: EventReader<HistoryCommands>,
    mut willo_query: Query<&mut WilloAnimationState>,
    audio: Res<Audio>,
    sfx: Res<AssetHolder>,
) {
    for command in history_commands.iter() {
        match command {
            HistoryCommands::Rewind | HistoryCommands::Reset => {
                *willo_query.single_mut() = WilloAnimationState::Idle(Direction::Down);
                audio.play(sfx.undo_sound.clone_weak());
            }
            _ => (),
        }
    }
}

fn play_death_animations(
    mut willo_query: Query<&mut WilloAnimationState>,
    mut death_event_reader: EventReader<ExorcismEvent>,
) {
    for ExorcismEvent { willo_entity } in death_event_reader.iter() {
        if let Ok(mut animation_state) = willo_query.get_mut(*willo_entity) {
            *animation_state = WilloAnimationState::Dying;
        }
    }
}

fn willo_input(
    mut willo_query: Query<&mut WilloState>,
    input: Res<Input<KeyCode>>,
    mut history_commands: EventWriter<HistoryCommands>,
    mut rewind_settings: ResMut<RewindSettings>,
    time: Res<Time>,
) {
    for mut willo in willo_query.iter_mut() {
        if *willo == WilloState::Waiting {
            if input.just_pressed(KeyCode::W) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::W)
            } else if input.just_pressed(KeyCode::A) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::A)
            } else if input.just_pressed(KeyCode::S) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::S)
            } else if input.just_pressed(KeyCode::D) {
                history_commands.send(HistoryCommands::Record);
                *willo = WilloState::RankMove(KeyCode::D)
            }
        }

        if *willo == WilloState::Waiting || *willo == WilloState::Dead {
            if input.just_pressed(KeyCode::Z) {
                history_commands.send(HistoryCommands::Rewind);
                *willo = WilloState::Waiting;
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
                        *willo = WilloState::Waiting;

                        timer.set_duration(Duration::from_millis(*velocity as u64));
                    }
                }
            } else if input.just_pressed(KeyCode::R) {
                history_commands.send(HistoryCommands::Reset);
                *willo = WilloState::Waiting;
            }
        }
    }
}
