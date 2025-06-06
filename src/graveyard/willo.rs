//! Plugin, components and events providing functionality for Willo, the player character.
use crate::{
    animation::{FromComponentAnimator, SpriteSheetAnimation},
    from_component::FromComponentSet,
    graveyard::{exorcism::ExorcismEvent, gravestone::GraveId, volatile::Volatile},
    history::{History, HistoryCommands, HistoryPlugin},
    sokoban::{Direction, PushEvent, PushTracker, SokobanBlock, SokobanSets},
    AssetHolder, GameState, UNIT_LENGTH,
};
use bevy::prelude::*;
use bevy_easings::*;
use bevy_ecs_ldtk::{prelude::*, utils::grid_coords_to_translation};
use std::time::Duration;

/// Sets used by Willo systems.
#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum WilloSets {
    /// TODO: replace this with graveyard-level label since this is no longer used in this module.
    Input,
}

/// Plugin providing functionality for Willo, the player character.
pub struct WilloPlugin;

impl Plugin for WilloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FromComponentAnimator::<WilloAnimationState>::new(),
            HistoryPlugin::<GridCoords, _>::run_in_state(GameState::Graveyard),
        ))
        // Systems with potential easing end/beginning collisions cannot be in CoreSet::Update
        // see https://github.com/vleue/bevy_easings/issues/23
        .add_systems(
            Update,
            (
                push_sugar
                    .run_if(not(in_state(GameState::AssetLoading)))
                    .run_if(on_event::<PushEvent>())
                    .before(FromComponentSet),
                play_exorcism_animaton
                    .run_if(not(in_state(GameState::AssetLoading)))
                    .run_if(on_event::<ExorcismEvent>()),
                history_sugar
                    .run_if(not(in_state(GameState::AssetLoading)))
                    .run_if(on_event::<HistoryCommands>()),
            ),
        )
        .add_systems(
            PostUpdate,
            push_translation
                .run_if(not(in_state(GameState::AssetLoading)))
                .before(SokobanSets::EaseMovement),
        )
        .register_ldtk_entity::<WilloBundle>("Willo");
    }
}

/// Component that marks Willo and keeps track of their state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Component, Default)]
pub enum WilloState {
    /// Willo is idle, waiting for input.
    #[default]
    Waiting,
    /// Willo is dead and cannot accept input.
    Dead,
    /// Willo is performing the first part of a grave action.
    ///
    /// This move is defined by the rank of the gravestone on the movement table.
    RankMove(GraveId),
    /// Willo is performing the second part of a grave action.
    ///
    /// This move is defined by the file of the gravestone on the movement table.
    FileMove(GraveId),
}

/// Component enumerating the possible states of Willo's animation.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Component)]
pub enum WilloAnimationState {
    /// Willo is Idle, and facing a particular direction.
    Idle(Direction),
    /// Willo is pushing a gravestione in a particular direction.
    Push(Direction),
    /// Willo is dying.
    Dying,
    /// Willo is invisible (post-death).
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
            Push(Up | UpLeft | UpRight) => 1..2,
            Push(Down | DownLeft | DownRight) => 11..12,
            Push(Left) => 21..22,
            Push(Right) => 31..32,
            Idle(Up | UpLeft | UpRight) => 40..47,
            Idle(Down | DownLeft | DownRight) => 50..57,
            Idle(Left) => 60..67,
            Idle(Right) => 70..77,
            Dying => 80..105,
            Idle(Zero) | Push(Zero) | None => 3..4,
        };

        let frame_timer = Timer::new(Duration::from_millis(150), TimerMode::Repeating);

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
        MovementTimer(Timer::from_seconds(MOVEMENT_SECONDS, TimerMode::Once))
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct WilloBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    history: History<GridCoords>,
    #[with(SokobanBlock::new_dynamic)]
    sokoban_block: SokobanBlock,
    push_tracker: PushTracker,
    willo_state: WilloState,
    movement_timer: MovementTimer,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    willo_animation_state: WilloAnimationState,
    volatile: Volatile,
    volatile_history: History<Volatile>,
}

fn push_sugar(
    mut commands: Commands,
    mut push_events: EventReader<PushEvent>,
    mut willo_query: Query<(Entity, &mut WilloAnimationState)>,
    sfx: Res<AssetHolder>,
) {
    let (willo_entity, mut animation_state) = willo_query.single_mut();
    for PushEvent { direction, .. } in push_events
        .read()
        .filter(|PushEvent { pusher, .. }| *pusher == willo_entity)
    {
        commands.spawn(AudioBundle {
            source: sfx.push_sound.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
        *animation_state = WilloAnimationState::Push(*direction);
    }
}

fn push_translation(
    mut commands: Commands,
    willo_query: Query<
        (Entity, &GridCoords, &Transform, &WilloAnimationState),
        Changed<WilloAnimationState>,
    >,
) {
    if let Ok((entity, &grid_coords, transform, animation_state)) = willo_query.get_single() {
        let xy = grid_coords_to_translation(grid_coords, IVec2::splat(UNIT_LENGTH))
            + match animation_state {
                WilloAnimationState::Push(direction) => IVec2::from(direction).as_vec2() * 5.,
                _ => Vec2::splat(0.),
            };

        commands.entity(entity).insert(transform.ease_to(
            Transform::from_xyz(xy.x, xy.y, transform.translation.z),
            EaseFunction::CubicOut,
            EasingType::Once {
                duration: std::time::Duration::from_millis(100),
            },
        ));
    }
}

fn history_sugar(
    mut commands: Commands,
    mut history_commands: EventReader<HistoryCommands>,
    mut willo_query: Query<&mut WilloAnimationState>,
    sfx: Res<AssetHolder>,
) {
    for command in history_commands.read() {
        match command {
            HistoryCommands::Rewind | HistoryCommands::Reset => {
                *willo_query.single_mut() = WilloAnimationState::Idle(Direction::Down);
                commands.spawn(AudioBundle {
                    source: sfx.undo_sound.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            _ => (),
        }
    }
}

fn play_exorcism_animaton(mut willo_query: Query<&mut WilloAnimationState>) {
    if let Ok(mut animation_state) = willo_query.get_single_mut() {
        *animation_state = WilloAnimationState::Dying;
    }
}
