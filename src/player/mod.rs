use bevy::prelude::Resource;


#[derive(Resource)]
pub struct Player {
    pub points: i32,
    pub balls_available: i32,
    pub balls_spawned: i32,
    pub balls_in_play: i32
}

impl Default for Player {
    fn default() -> Self {
        Player {
            points: 0,
            balls_available: 0,
            balls_spawned: 0,
            balls_in_play: 0,
        }
    }
}

impl Player {

    pub fn reset(&mut self) {
        self.points = 0;
        self.balls_available = 0;
        self.balls_spawned = 0;
        self.balls_in_play = 0;
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
        if self.balls_spawned > 0 {
            self.balls_spawned -= 1;
            self.balls_in_play += 1;
        }
    }

    pub fn ball_lost(&mut self) {
        if self.balls_in_play > 0 {
            self.balls_in_play -= 1;
        }
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