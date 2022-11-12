use crate::{
    gameplay::components::*,
    gameplay::{DeathEvent, GoalEvent},
    gravestone::Gravestone,
    level_transition::{schedule_level_card, LevelCardEvent},
    willo::WilloState,
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

pub fn check_death(
    mut willo_query: Query<(Entity, &GridCoords, &mut WilloState)>,
    exorcism_query: Query<(Entity, &GridCoords), With<ExorcismBlock>>,
    mut death_event_writer: EventWriter<DeathEvent>,
) {
    if let Ok((entity, coords, mut willo)) = willo_query.get_single_mut() {
        if *willo != WilloState::Dead && exorcism_query.iter().any(|(_, g)| *g == *coords) {
            *willo = WilloState::Dead;
            death_event_writer.send(DeathEvent {
                willo_entity: entity,
            });
        }
    }
}

pub fn check_goal(
    mut commands: Commands,
    mut goal_query: Query<(Entity, &mut Goal, &GridCoords), With<Goal>>,
    block_query: Query<(Entity, &GridCoords), With<Gravestone>>,
    mut goal_events: EventWriter<GoalEvent>,
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
        commands.insert_resource(NextState(GameState::LevelTransition));

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
