use bevy::prelude::Resource;
use crate::powerups::PowerUpType;


pub enum PlayerState {
    Open,
    HasWon,
    HasLost,
}

#[derive(Resource)]
pub struct Player {
    pub state: PlayerState,
    pub points: i32,
    pub balls_available: i32,
    pub balls_spawned: i32,
    pub balls_in_play: i32,
    pub balls_lost: i32,
    pub balls_grabbed: i32,
    pub power_ups: Vec<PowerUpType>

}

impl Default for Player {
    fn default() -> Self {
        Player {
            state: PlayerState::Open,
            points: 0,
            balls_available: 0,
            balls_spawned: 0,
            balls_in_play: 0,
            balls_grabbed: 0,
            balls_lost: 0,
            power_ups: vec![]
        }
    }
}

impl Player {

    pub fn reset(&mut self) {
        self.state = PlayerState::Open;
        self.points = 0;
        self.balls_available = 0;
        self.balls_spawned = 0;
        self.balls_in_play = 0;
        self.balls_grabbed = 0;
        self.balls_lost = 0;
        self.power_ups = vec![];
    }

    pub fn set_balls(&mut self, count: i32) {
        self.balls_available = count;
    }

    pub fn ball_spawned(&mut self) {
        if self.balls_available > 0 {
            self.balls_available -= 1;
            self.balls_spawned += 1;
        }
    }

    pub fn ball_launched(&mut self) {
        if self.balls_grabbed > 0 {
            self.balls_grabbed -= 1;
            self.balls_in_play += 1;
        } else if self.balls_spawned > 0 {
            self.balls_spawned -= 1;
            self.balls_in_play += 1;
        }
    }

    pub fn ball_grabbed(&mut self) {
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