use std::time::Duration;
use bevy::prelude::Resource;
use crate::block::{BlockBehaviour, BlockType};


pub enum BlockHitType {
    Regular,
    DirectHit
}

#[derive(Resource)]
pub struct MatchState {

    // Points achieved in the level
    pub points: i32,

    // Blocks present in the level
    pub blocks: i32,

    // Number of times the ball hit the paddle
    pub paddle_bounces: i32,

    // Number of times the ball hit a wall
    pub wall_hits: i32,

    // Was the last ball contact a paddle bounce?
    pub direct_hit_possible: bool,

    // Number of times a block was hit without hitting a wall first. Multiple blocks after a
    // paddle bounce give only one direct hit each TODO: Test this out. Maybe only one hit
    pub direct_hits: i32,

    // Number of times at least one block hit after a paddle bounce
    pub paddle_bounce_combo: i32,
    pub paddle_bounce_combo_possible: bool,

    // Number of blocks after one paddle bounce
    pub single_bounce_combo: i32,

    pub balls_lost: i32,

    pub balls_used: i32,

    pub balls: i32,

    pub time_taken: Duration,

    pub blocks_hit: i32,
    pub blocks_lost: i32
}


impl MatchState {
    pub fn reset(&mut self) {
        self.blocks = 0;
        self.paddle_bounces = 0;
        self.points = 0;
        self.wall_hits = 0;
        self.direct_hit_possible = false;
        self.paddle_bounce_combo_possible = false;
        self.paddle_bounce_combo = 0;
        self.single_bounce_combo = 0;
        self.direct_hits = 0;
        self.balls_used = 0;
        self.balls_lost = 0;
        self.time_taken = Duration::default();
        self.balls = 0;
        self.blocks_hit = 0;
        self.blocks_lost = 0;
    }
}

impl Default for MatchState {
    fn default() -> Self {
        MatchState {
            blocks: 0,
            points: 0,
            paddle_bounces: 0,
            wall_hits: 0,
            direct_hit_possible: false,
            direct_hits: 0,
            paddle_bounce_combo: 0,
            single_bounce_combo: 0,
            balls_lost: 0,
            balls_used: 0,
            time_taken: Default::default(),
            blocks_hit: 0,
            paddle_bounce_combo_possible: false,
            balls: 0,
            blocks_lost: 0,
        }
    }
}


impl MatchState {

    pub fn add_paddle_bounce(&mut self) {
        self.paddle_bounces += 1;
        self.direct_hit_possible = true;

        if self.paddle_bounce_combo_possible {
            self.paddle_bounce_combo = 0;
            self.paddle_bounce_combo_possible = false;
        } else {
            self.paddle_bounce_combo_possible = true;
        }

        self.single_bounce_combo = 0;
    }

    pub fn ball_launched(&mut self) {
        self.direct_hit_possible = false;
        self.paddle_bounce_combo = 0;
        self.paddle_bounce_combo_possible = true;
    }



    fn get_base_points( block_type: &BlockType, behaviour: &BlockBehaviour) -> i32 {
        let mut base_points = match block_type {
            BlockType::Simple => 100,
            BlockType::Hardling => 150,
            BlockType::Concrete => 200,
            _ => 0
        };

        base_points += match behaviour {
            BlockBehaviour::SittingDuck => 0,
            BlockBehaviour::Spinner => 50,
            BlockBehaviour::Vanisher => 100,
            BlockBehaviour::Repuslor => 150,
            BlockBehaviour::EvaderR(_) | BlockBehaviour::EvaderL(_) | BlockBehaviour::EvaderD(_) | BlockBehaviour::EvaderU(_) => 150,
        };

        base_points
    }

    // Only when block removed
    pub fn add_block_hit(&mut self, block_type: &BlockType, behaviour: &BlockBehaviour) -> (BlockHitType, i32) {
        let mut awarded = 0;
        let mut hit_type = BlockHitType::Regular;
        self.blocks -= 1;
        self.blocks_hit += 1;

        let base_points = MatchState::get_base_points(block_type, behaviour);
        awarded += base_points * (1 + self.paddle_bounce_combo) + 10 * self.single_bounce_combo;

        self.single_bounce_combo += 1;

        if self.paddle_bounce_combo_possible {
            self.paddle_bounce_combo += 1;
            self.paddle_bounce_combo_possible = false;

        }

        if self.direct_hit_possible {
            self.direct_hits += 1;

            awarded += base_points * self.paddle_bounce_combo;

            self.direct_hit_possible = false;

            hit_type = BlockHitType::DirectHit;
        }

        self.points += awarded;

        (hit_type, awarded)

    }

    pub fn get_combos(&self) -> (i32, i32) {
        (self.single_bounce_combo, self.paddle_bounce_combo)
    }

    pub fn add_wall_hit(&mut self) {
        self.wall_hits += 1;
        self.direct_hit_possible = false;
    }

    pub fn ball_lost(&mut self) {
        self.direct_hit_possible = false;
        self.paddle_bounce_combo = 0;
        self.single_bounce_combo = 0;
    }

    pub fn block_lost(&mut self) {
        self.blocks_lost += 1;
        self.blocks -= 1;
    }

    pub fn set_block_count(&mut self, count: i32) {
        self.blocks = count;
    }

    pub fn set_ball_count(&mut self, count: i32) {
        self.balls = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_paddle_bounces() {

        let mut s = MatchState::default();

        s.add_paddle_bounce();
        s.add_paddle_bounce();
        s.add_paddle_bounce();

        assert_eq!(s.paddle_bounces, 3);
    }

    #[test]
    fn counts_a_direct_hit() {
        let mut s = MatchState::default();
        s.blocks = 100;

        s.add_paddle_bounce();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);
        s.add_paddle_bounce();
        s.add_wall_hit();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);
        s.add_paddle_bounce();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);

        assert_eq!(s.direct_hits, 2);
        assert_eq!(s.blocks, 97);
        assert_eq!(s.wall_hits, 1);
        assert_eq!(s.paddle_bounces, 3);
    }


    // Number of blocks after one paddle bounce
    #[test]
    fn handles_single_bounce_combo() {
        let mut s = MatchState::default();
        s.blocks = 100;

        s.add_paddle_bounce();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);

        assert_eq!(s.single_bounce_combo, 3);

        s.add_wall_hit();
        assert_eq!(s.single_bounce_combo, 3);

        s.add_paddle_bounce();
        assert_eq!(s.single_bounce_combo, 0);
    }


    // Number of times at least one block hit after a paddle bounce
    #[test]
    fn handles_paddle_bounce_combo() {
        let mut s = MatchState::default();
        s.blocks = 100;

        s.add_paddle_bounce();
        s.add_wall_hit();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);
        s.add_wall_hit();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);

        assert_eq!(s.paddle_bounce_combo, 1);

        s.add_paddle_bounce();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);

        assert_eq!(s.paddle_bounce_combo, 2);

        s.add_paddle_bounce();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);

        assert_eq!(s.paddle_bounce_combo, 3);

        s.add_paddle_bounce();
        s.add_paddle_bounce();

        assert_eq!(s.paddle_bounce_combo, 0);

        s.add_paddle_bounce();
        s.add_block_hit(&BlockType::Simple, &BlockBehaviour::SittingDuck);
        assert_eq!(s.paddle_bounce_combo, 1);
    }



}