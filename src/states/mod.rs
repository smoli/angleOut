use bevy::math::Vec3;
use bevy::prelude::Resource;
use bevy_rapier2d::prelude::Real;

#[derive(Resource)]
pub struct PaddleState {
    pub paddle_rotation: Real,
    pub paddle_position: Vec3
}

#[derive(Resource)]
pub struct GameState {
    pub running: bool,
    pub blocks: usize,
    pub points: u32
}

impl GameState {
    pub fn addBlocks(&mut self, count: usize) {
        self.blocks += count;
    }

    pub fn subBlocks(&mut self, count: usize) {
        if self.blocks >= count {
            self.blocks -= count;
        } else {
            self.blocks = 0;
            println!("Block count underflow!");
        }
    }
}

