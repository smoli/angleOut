use std::collections::HashMap;
use bevy::log::info;
use bevy::prelude::{Component, Entity};

use crate::powerups::{PowerUpData, PowerUpType};


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
    pub balls_grabbed: i32
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
            balls_lost: 0
        }
    }
}

impl Player {

    pub fn reset(&mut self) {
        self.state = PlayerState::Open;
        self.points = 0;
        self.balls_available = 0;
        self.balls_carried = 0;
        self.balls_in_play = 0;
        self.balls_grabbed = 0;
        self.balls_lost = 0;
    }

    pub fn set_balls(&mut self, count: i32) {
        self.balls_available = count;
    }

    pub fn ball_spawned(&mut self) {
        if self.balls_available > 0 {
            info!("Ball spawned");
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

    pub fn has_player_lost(&self) -> bool {
        self.balls_available + self.balls_in_play == 0
    }
}