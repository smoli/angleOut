use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Query, SystemSet};

use crate::player::{Player, PlayerState};
use crate::state::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(game_start)
            )

        ;
    }
}

fn game_start(
    mut commands: Commands,
    mut players:Query<&mut Player>,
) {
    let player = players.get_single_mut();

    match player {
        Ok(mut player) => {
            player.reset();
            player.set_balls(3);
        }
        Err(_) => {
            commands
                .spawn(Player {
                    state: PlayerState::Open,
                    points: 0,
                    balls_available: 3,
                    balls_spawned: 0,
                    balls_in_play: 0,
                    balls_lost: 0,
                    balls_grabbed: 0,
                });
        }
    }

    //player.power_ups.insert(PowerUpType::Bouncer, Bouncer { bounces: 3 });
}

