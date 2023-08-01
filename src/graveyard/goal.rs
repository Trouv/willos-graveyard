//! Plugin providing functionality for goal tiles with victory logic and goal ghost visuals.
use crate::{
    graveyard::{exorcism::ExorcismSets, gravestone::GraveId},
    level_transition::TransitionTo,
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use rand::Rng;
use std::{ops::Range, time::Duration};

/// Plugin providing functionality for goal tiles with victory logic and goal ghost visuals.
pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GoalEvent>()
            .init_resource::<GoalGhostSettings>()
            .add_systems(
                Update,
                (
                    spawn_goal_ghosts.run_if(in_state(GameState::LevelTransition)),
                    check_goal
                        .run_if(in_state(GameState::Graveyard))
                        .after(ExorcismSets::CheckDeath),
                    goal_ghost_animation.run_if(not(in_state(GameState::AssetLoading))),
                    goal_ghost_event_sugar.run_if(not(in_state(GameState::AssetLoading))),
                ),
            )
            .register_ldtk_entity::<GoalBundle>("Goal");
    }
}

/// Component that marks goal tiles and stores whether or not it is currently "satisfied".
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct Goal {
    met: bool,
}

/// Event that fires when a goal's state changes.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GoalEvent {
    Met {
        goal_entity: Entity,
        stone_entity: Entity,
    },
    UnMet {
        goal_entity: Entity,
    },
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
struct GoalBundle {
    #[grid_coords]
    grid_coords: GridCoords,
    goal: Goal,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
}

/// Resource for defining the visual behavior of goal ghosts.
#[derive(Clone, Eq, PartialEq, Debug, Hash, Resource)]
struct GoalGhostSettings {
    no_turn_length: Range<usize>,
    turn_length: Range<usize>,
    no_blink_length: Range<usize>,
    blink_length: Range<usize>,
    frame_duration: Duration,
    idle_frame_count: usize,
    happy_frame_count: usize,
    none_frame_index: usize,
    num_columns: usize,
    num_rows: usize,
    atlas: Option<Handle<TextureAtlas>>,
}

impl Default for GoalGhostSettings {
    fn default() -> Self {
        GoalGhostSettings {
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
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum HandDirection {
    Right,
    Left,
}

/// Component defining the current abstract state of the goal ghost animation.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GoalAnimationState {
    #[default]
    Idle,
    Turn {
        hand: HandDirection,
        frames: usize,
    },
    Blinking {
        frames: usize,
    },
    Happy {
        frame: usize,
    },
    None,
}

/// Component tracking the goal ghost animation in detail.
#[derive(Clone, Debug, Component)]
struct GoalGhostAnimation {
    goal_entity: Entity,
    frame_timer: Timer,
    column: usize,
    frames_since_blink: usize,
    frames_since_turn: usize,
    state: GoalAnimationState,
}

impl GoalGhostAnimation {
    fn new(goal_entity: Entity, frame_timer: Timer) -> GoalGhostAnimation {
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

/// Utility for providing uniform probability over a range for a particular point in that range.
///
/// It behaves as if a random number was chosen within the range uniformly, but can be queried at
/// any point in that range as if all the points below it weren't chosen.
/// So, if the point chosen is near the end of the range, the resulting probability is higher,
/// since the choice must have occurred *somewhere* in the range and it hasn't occurred yet.
fn range_chance(range: &Range<usize>, current: usize) -> f32 {
    ((current as f32 - range.start as f32) / (range.end as f32 - range.start as f32)).clamp(0., 1.)
}

fn check_goal(
    mut commands: Commands,
    mut goal_query: Query<(Entity, &mut Goal, &GridCoords), With<Goal>>,
    block_query: Query<(Entity, &GridCoords), With<GraveId>>,
    mut goal_events: EventWriter<GoalEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    level_selection: Res<LevelSelection>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    audio: Res<Audio>,
    asset_holder: Res<AssetHolder>,
) {
    // If the goal is not loaded for whatever reason (for example when hot-reloading levels),
    // the goal will automatically be "met", loading the next level.
    // This if statement prevents that.
    if goal_query.iter().count() == 0 {
        return;
    }

    let mut level_goal_met = true;

    for (goal_entity, mut goal, goal_grid_coords) in goal_query.iter_mut() {
        let mut goal_met = false;
        for (stone_entity, block_grid_coords) in block_query.iter() {
            if goal_grid_coords == block_grid_coords {
                goal_met = true;

                if !goal.met {
                    goal.met = true;

                    goal_events.send(GoalEvent::Met {
                        stone_entity,
                        goal_entity,
                    });
                }

                break;
            }
        }
        if !goal_met {
            level_goal_met = false;

            if goal.met {
                goal_events.send(GoalEvent::UnMet { goal_entity });
                goal.met = false;
            }
        }
    }

    if level_goal_met {
        next_state.set(GameState::LevelTransition);

        if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
            if let Some((level_index, _)) = ldtk_asset
                .iter_levels()
                .enumerate()
                .find(|(i, level)| level_selection.is_match(i, level))
            {
                // Currently this doesn't have a time buffer like it used to.
                // This will change as we make a more elaborate level transition workflow.
                commands.insert_resource(TransitionTo(LevelSelection::Index(level_index + 1)));
            }
        }

        audio.play(asset_holder.victory_sound.clone_weak());
    }
}
fn goal_ghost_event_sugar(
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

fn goal_ghost_animation(
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

fn spawn_goal_ghosts(
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
                    None,
                    None,
                );
                let atlas_handle = texture_atlases.add(texture_atlas);

                goal_ghost_settings.atlas = Some(atlas_handle.clone());
                atlas_handle.clone()
            }
        };

        let ghost_entity = commands
            .spawn(SpriteSheetBundle {
                texture_atlas: atlas_handle,
                transform: Transform::from_xyz(0., 1., 2.5),
                ..default()
            })
            .insert(GoalGhostAnimation::new(
                goal_entity,
                Timer::new(goal_ghost_settings.frame_duration, TimerMode::Repeating),
            ))
            .id();

        commands.entity(goal_entity).add_child(ghost_entity);
    }
}
