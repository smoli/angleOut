use bevy::prelude::Resource;

#[derive(Resource)]
pub struct MatchState {
    pub running: bool,
    pub blocks: usize,
    pub paddle_bounces: usize,
    pub points: u32,
    pub balls_available: usize,
    pub balls_active: usize,
}


impl MatchState {
    pub fn reset(&mut self) {
        self.running = false;
        self.blocks = 0;
        self.paddle_bounces = 0;
        self.points = 0;
        self.balls_available = 2;
        self.balls_active = 0;
    }
}