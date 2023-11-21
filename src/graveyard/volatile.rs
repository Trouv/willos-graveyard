//! Plugin providing the core logic for [`Volatile`] entities.
//!
//! Volatile entities are "Solid" initially.
//! Once they come into contact with another Volatile entity - they are both "Sublimated".
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    history::{FlushHistoryCommands, HistoryPlugin},
    utils::any_match_filter,
    GameState,
};

/// Plugin providing the core logic for [`Volatile`] entities.
///
/// Volatile entities are "Solid" initially.
/// Once they come into contact with another Volatile entity - they are both "Sublimated".
pub struct VolatilePlugin;

/// `SystemSet` performing sublimation of [`Volatile`] entities.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Sublimation;

impl Plugin for VolatilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HistoryPlugin::<Volatile, _>::run_in_state(
            GameState::Graveyard,
        ))
        .add_systems(
            Update,
            sublimation
                .run_if(in_state(GameState::Graveyard))
                .run_if(any_match_filter::<(With<Volatile>, Changed<GridCoords>)>)
                .after(FlushHistoryCommands)
                .in_set(Sublimation),
        );
    }
}

/// Component defining the volatility of an entity and its volatile state.
///
/// If two volatile solids share the same [`GridCoords`] space, they both are sublimated.
/// What this means for a particular entity should be defined elsewhere.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Component)]
pub enum Volatile {
    /// The entity is still a volatile solid.
    #[default]
    Solid,
    /// The entity has collided with another volatile solid, and has been sublimated.
    Sublimated,
}

impl Volatile {
    /// Returns `true` if this instance is [`Volatile::Solid`].
    pub fn is_solid(&self) -> bool {
        matches!(self, Volatile::Solid)
    }

    /// Sets this instance to [`Volatile::Sublimated`].
    pub fn sublimate(&mut self) {
        *self = Volatile::Sublimated;
    }
}

/// System performing sublimation logic.
///
/// Obtains a separate query for *moving* volatiles and *all* volatiles.
/// This is so it can split moving volatiles and stationary volatiles into separate collections.
/// This allows us to limit our collision detection to checking moved-volatiles against all-volatiles,
/// rather than all-volatiles against all-volatiles.
fn sublimation(
    moved_volatile_entities: Query<(), (With<Volatile>, Changed<GridCoords>)>,
    mut all_volatiles: Query<(Entity, &GridCoords, &mut Volatile)>,
) {
    // Split volatiles into moved and stationary collections.
    let (mut moved_volatiles, mut stationary_volatiles): (Vec<_>, Vec<_>) = all_volatiles
        .iter_mut()
        .partition(|(entity, ..)| moved_volatile_entities.contains(*entity));

    // Check for collisions between moved volatiles.
    for index in 0..moved_volatiles.len() - 1 {
        if let [(_, grid_coords_a, volatile_a), remaining_moved_volatiles @ ..] =
            &mut moved_volatiles[index..]
        {
            if volatile_a.is_solid() {
                for (_, grid_coords_b, volatile_b) in remaining_moved_volatiles.iter_mut() {
                    if volatile_b.is_solid() && grid_coords_a == grid_coords_b {
                        volatile_a.sublimate();
                        volatile_b.sublimate();
                    }
                }
            }
        }
    }

    // Check for collisions between moved volatiles and stationary volatiles.
    for (_, grid_coords_a, volatile_a) in moved_volatiles.iter_mut() {
        if volatile_a.is_solid() {
            for (_, grid_coords_b, volatile_b) in stationary_volatiles.iter_mut() {
                if volatile_b.is_solid() && grid_coords_a == grid_coords_b {
                    volatile_a.sublimate();
                    volatile_b.sublimate();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_setup() -> App {
        let mut app = App::new();

        app.add_state::<GameState>().add_plugins(VolatilePlugin);
        app.world
            .insert_resource(NextState(Some(GameState::Graveyard)));
        app.update();

        app
    }

    struct SpawnVolatilesParams<const N: usize> {
        volatiles: [(GridCoords, Volatile); N],
    }

    impl<const N: usize> SpawnVolatilesParams<N> {
        fn new() -> SpawnVolatilesParams<N> {
            let mut volatiles: [(GridCoords, Volatile); N] = [Default::default(); N];
            for i in 0..N {
                volatiles[i].0.y = i as i32;
            }

            SpawnVolatilesParams { volatiles }
        }

        fn spawn(self, app: &mut App) -> [Entity; N] {
            let entities = self
                .volatiles
                .map(|volatile_bundle| app.world.spawn(volatile_bundle).id());

            app.update();

            entities
        }
    }

    #[test]
    fn solid_moving_onto_solid_sublimates_both() {
        let mut app = app_setup();

        app.update();

        let [bottom, _] = SpawnVolatilesParams::<2>::new().spawn(&mut app);

        assert!(app
            .world
            .query::<&Volatile>()
            .iter(&app.world)
            .all(|volatile| volatile == &Volatile::Solid));

        app.world.get_mut::<GridCoords>(bottom).unwrap().y += 1;

        app.update();

        assert!(app
            .world
            .query::<&Volatile>()
            .iter(&app.world)
            .all(|volatile| volatile == &Volatile::Sublimated));
    }

    #[test]
    fn solid_moving_not_on_solid_doesnt_sublimate() {
        let mut app = app_setup();

        app.update();

        let [bottom, _] = SpawnVolatilesParams::<2>::new().spawn(&mut app);

        app.world.get_mut::<GridCoords>(bottom).unwrap().y -= 1;

        app.update();

        assert!(app
            .world
            .query::<&Volatile>()
            .iter(&app.world)
            .all(|volatile| volatile == &Volatile::Solid));
    }

    #[test]
    fn sublimated_moving_onto_solid_doesnt_sublimate() {
        let mut app = app_setup();

        app.update();

        let mut spawn_params = SpawnVolatilesParams::<2>::new();
        spawn_params.volatiles[0].1 = Volatile::Sublimated;
        let [bottom, top] = spawn_params.spawn(&mut app);

        app.world.get_mut::<GridCoords>(bottom).unwrap().y += 1;

        app.update();

        assert_eq!(
            app.world.get::<Volatile>(bottom).unwrap(),
            &Volatile::Sublimated
        );
        assert_eq!(app.world.get::<Volatile>(top).unwrap(), &Volatile::Solid);
    }

    #[test]
    fn solid_moving_onto_sublimated_doesnt_sublimate() {
        let mut app = app_setup();

        app.update();

        let mut spawn_params = SpawnVolatilesParams::<2>::new();
        spawn_params.volatiles[1].1 = Volatile::Sublimated;
        let [bottom, top] = spawn_params.spawn(&mut app);

        app.world.get_mut::<GridCoords>(bottom).unwrap().y += 1;

        app.update();

        assert_eq!(app.world.get::<Volatile>(bottom).unwrap(), &Volatile::Solid);
        assert_eq!(
            app.world.get::<Volatile>(top).unwrap(),
            &Volatile::Sublimated
        );
    }

    #[test]
    fn sublimated_moving_onto_sublimated_keeps_sublimated() {
        let mut app = app_setup();

        app.update();

        let mut spawn_params = SpawnVolatilesParams::<2>::new();
        spawn_params.volatiles = spawn_params
            .volatiles
            .map(|(grid_coords, _)| (grid_coords, Volatile::Sublimated));
        let [bottom, _] = spawn_params.spawn(&mut app);

        app.world.get_mut::<GridCoords>(bottom).unwrap().y += 1;

        app.update();

        assert!(app
            .world
            .query::<&Volatile>()
            .iter(&app.world)
            .all(|volatile| volatile == &Volatile::Sublimated));
    }

    #[test]
    fn solids_moving_into_eachother_sublimates_both() {
        let mut app = app_setup();

        app.update();

        let [bottom, top] = SpawnVolatilesParams::<2>::new().spawn(&mut app);

        *app.world.get_mut::<GridCoords>(bottom).unwrap() = GridCoords::new(10, 10);
        *app.world.get_mut::<GridCoords>(top).unwrap() = GridCoords::new(10, 10);

        app.update();

        assert!(app
            .world
            .query::<&Volatile>()
            .iter(&app.world)
            .all(|volatile| volatile == &Volatile::Sublimated));
    }

    #[test]
    fn solid_and_sublimated_moving_into_eachother_doesnt_sublimate() {
        let mut app = app_setup();

        app.update();

        let mut spawn_params = SpawnVolatilesParams::<2>::new();
        spawn_params.volatiles[0].1 = Volatile::Sublimated;
        let [bottom, top] = spawn_params.spawn(&mut app);

        *app.world.get_mut::<GridCoords>(bottom).unwrap() = GridCoords::new(10, 10);
        *app.world.get_mut::<GridCoords>(top).unwrap() = GridCoords::new(10, 10);

        app.update();

        assert_eq!(
            app.world.get::<Volatile>(bottom).unwrap(),
            &Volatile::Sublimated
        );
        assert_eq!(app.world.get::<Volatile>(top).unwrap(), &Volatile::Solid);
    }

    #[test]
    fn sublimateds_moving_eachother_keeps_sublimated() {
        let mut app = app_setup();

        app.update();

        let mut spawn_params = SpawnVolatilesParams::<2>::new();
        spawn_params.volatiles = spawn_params
            .volatiles
            .map(|(grid_coords, _)| (grid_coords, Volatile::Sublimated));
        let [bottom, top] = spawn_params.spawn(&mut app);

        *app.world.get_mut::<GridCoords>(bottom).unwrap() = GridCoords::new(10, 10);
        *app.world.get_mut::<GridCoords>(top).unwrap() = GridCoords::new(10, 10);

        app.update();

        assert!(app
            .world
            .query::<&Volatile>()
            .iter(&app.world)
            .all(|volatile| volatile == &Volatile::Sublimated));
    }

    #[test]
    fn solids_moving_onto_solid_sublimates_movers() {
        let mut app = app_setup();

        app.update();

        let [bottom, middle, top] = SpawnVolatilesParams::<3>::new().spawn(&mut app);

        *app.world.get_mut::<GridCoords>(bottom).unwrap() = GridCoords::new(0, 1);
        *app.world.get_mut::<GridCoords>(top).unwrap() = GridCoords::new(0, 1);

        app.update();

        assert_eq!(
            app.world.get::<Volatile>(bottom).unwrap(),
            &Volatile::Sublimated
        );
        assert_eq!(app.world.get::<Volatile>(middle).unwrap(), &Volatile::Solid);
        assert_eq!(
            app.world.get::<Volatile>(top).unwrap(),
            &Volatile::Sublimated
        );
    }

    #[test]
    fn solid_and_sublimated_moving_onto_solid_sublimates_remaining() {
        let mut app = app_setup();

        app.update();

        let mut spawn_params = SpawnVolatilesParams::<3>::new();
        spawn_params.volatiles[0].1 = Volatile::Sublimated;
        let [bottom, _, top] = spawn_params.spawn(&mut app);

        *app.world.get_mut::<GridCoords>(bottom).unwrap() = GridCoords::new(0, 1);
        *app.world.get_mut::<GridCoords>(top).unwrap() = GridCoords::new(0, 1);

        app.update();

        assert!(app
            .world
            .query::<&Volatile>()
            .iter(&app.world)
            .all(|volatile| volatile == &Volatile::Sublimated));
    }
}
