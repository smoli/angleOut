use bevy::app::{App, Plugin};
use bevy::prelude::{default, Commands, Entity, OnEnter, Query, ResMut};
use crate::level::Levels;

use crate::player::Player;
use crate::powerups::{Bouncer, Grabber};
use crate::state::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), game_start)

        ;
    }
}

/// Starts a new game: back to the first level, with a player who has three
/// balls and nothing behind them.
///
/// The menu is a clean slate, which includes anything the editor was in the
/// middle of: a playtest that somehow ended up here rather than back in the
/// editor would otherwise leave the campaign shadowed by a level the player
/// never asked for - see [`Levels::stop_playtest`].
fn game_start(
    mut commands: Commands,
    mut players:Query<(Entity, &mut Player)>,
    mut levels: ResMut<Levels>
) {
    let player = players.single_mut();

    levels.current_level = 0;
    levels.stop_playtest();

    match player {
        Ok((entity, mut player)) => {

            player.reset();
            player.balls_available = 3;

            commands.entity(entity)
                .insert(Bouncer {
                    bounces: -1,
                });
        }
        Err(_) => {
            commands
                .spawn(Player {
                    balls_available: 3,
                    ..default()
                })
                .insert(Bouncer {
                    bounces: -1
                })
                .insert(Grabber {
                    grabs: 5,
                })
            ;
        }
    }

    //player.power_ups.insert(PowerUpType::Bouncer, Bouncer { bounces: 3 });
}


#[cfg(test)]
mod tests {
    use super::*;

    use bevy::app::App;
    use bevy::prelude::{Handle, NextState, ResMut};
    use bevy::state::app::{AppExtStates, StatesPlugin};
    use bevy::MinimalPlugins;

    /// Whatever the game was doing before, the menu starts it over - including
    /// `c0013`'s playtest, which has no business outliving the editor.
    #[test]
    fn a_new_game_starts_from_the_top_of_the_campaign() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_state(GameState::Start);
        app.add_plugins(GamePlugin);

        let mut levels = Levels { current_level: 3, ..default() };
        levels.start_playtest(Handle::default());
        app.insert_resource(levels);

        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::InGame);
        app.update();

        let levels = app.world().resource::<Levels>();
        assert_eq!(levels.current_level, 0);
        assert!(!levels.is_playtesting());
    }
}
