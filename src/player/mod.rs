use std::collections::HashMap;
use bevy::app::App;
use bevy::log::info;
use bevy::prelude::{Component, Entity, Plugin, Query, SystemSet, With, Without, Commands};

use crate::pickups::{Pickup, PickupType};

use crate::powerups::{Grabber};
use crate::state::GameState;

pub enum PlayerState {
    Open,
    HasWon,
    HasLost,
}

#[derive(Component)]
pub struct Player {
    pub state: PlayerState,
    pub points: i32,
    pub balls_available: i32,
    pub balls_carried: i32,
    pub balls_in_play: i32,
    pub balls_lost: i32,
    pub balls_grabbed: i32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            state: PlayerState::Open,
            points: 0,
            balls_available: 0,
            balls_carried: 0,
            balls_in_play: 0,
            balls_grabbed: 0,
            balls_lost: 0,
        }
    }
}

impl Player {
    pub fn reset(&mut self) {
        //info!("Player reset");
        self.state = PlayerState::Open;
        self.points = 0;
        self.balls_available = 0;
        self.balls_carried = 0;
        self.balls_in_play = 0;
        self.balls_grabbed = 0;
        self.balls_lost = 0;
    }

    pub fn reset_for_match(&mut self) {
        self.balls_lost = 0;
        self.balls_in_play = 0;
        self.balls_grabbed = 0;
        self.balls_carried = 0;
        self.state = PlayerState::Open;
    }

    pub fn set_balls(&mut self, count: i32) {
        self.balls_available = count;
    }

    pub fn add_balls(&mut self, count: i32) {
        self.balls_available += count
    }

    pub fn ball_spawned(&mut self) {
        if self.balls_available > 0 {
            //info!("Ball spawned");
            self.balls_available -= 1;
            self.balls_carried += 1;
        }
    }

    pub fn ball_launched(&mut self) {
        if self.balls_grabbed > 0 {
            self.balls_grabbed -= 1;
        }

        if self.balls_carried > 0 {
            self.balls_carried -= 1;
            self.balls_in_play += 1;
        }
    }

    pub fn ball_grabbed(&mut self) {
        self.balls_carried += 1;
        self.balls_grabbed += 1;
        self.balls_in_play -= 1;
    }

    pub fn ball_lost(&mut self) {
        if self.balls_in_play > 0 {
            self.balls_in_play -= 1;
        }
        self.balls_lost += 1;
    }

    pub fn player_has_won(&mut self, match_points: i32) {
        self.balls_available += self.balls_in_play;
        self.balls_in_play = 0;
        self.points += match_points;
    }

    pub fn total_ball_count(&self) -> i32 {
        return self.balls_carried + self.balls_in_play + self.balls_grabbed + self.balls_available;
    }

}


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(player_pickup_grabber)
                    .with_system(player_pickup_more_balls)
            )

        ;
    }
}


fn player_pickup_more_balls(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player, &Pickup)>
) {
    for (entity, mut player, pickup) in &mut players {
        if let PickupType::MoreBalls(count) = pickup.pickup_type {
            player.add_balls(count);
            commands.entity(entity)
                .remove::<Pickup>();
        }
    }
}


fn player_pickup_grabber(
    mut commands: Commands,
    with_grabber: Query<(Entity, &Grabber, &Pickup), With<Player>>,
    without_grabber: Query<(Entity, &Pickup), (Without<Grabber>, With<Player>)>,
) {
    if let Ok((entity, grabber, pickup)) = with_grabber.get_single() {
        if let PickupType::Grabber(count) = pickup.pickup_type {
            commands.entity(entity)
                .remove::<Pickup>()
                .insert(Grabber {
                    grabs: count + grabber.grabs
                });

            //info!("More grabs!")
        }
    } else if let Ok((entity, pickup)) = without_grabber.get_single() {
        if let PickupType::Grabber(count) = pickup.pickup_type {
            commands.entity(entity)
                .remove::<Pickup>()
                .insert(Grabber {
                    grabs: count
                });
            //info!("Finally grabs!")
        }
    }
}