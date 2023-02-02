use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Entity, Query, SystemSet};
use bevy::utils::default;

use crate::player::{Player};
use crate::powerups::{Bouncer, Grabber};
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
    mut players:Query<(Entity, &mut Player)>,
) {
    let player = players.get_single_mut();

    match player {
        Ok((entity, mut player)) => {
            player.reset();
            player.set_balls(3);

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

