use bevy::prelude::Resource;



#[derive(Resource)]
pub struct MatchState {

    // Blocks present in the level
    pub blocks: usize,

    // Points achieved in the level
    pub points: u32,

    // Number of times the ball hit the paddle
    pub paddle_bounces: usize,

    // Number of times the ball hit a wall
    pub wall_bounces: usize,

    // Was the last ball contact a paddle bounce?
    pub direct_hit_possible: bool,

    // Number of times a block was hit without hitting a wall first. Multiple blocks after a
    // paddle bounce give only one direct hit each TODO: Test this out. Maybe only one hit
    pub direct_hits: usize,

    // Number of times at least one block hit after a paddle bounce
    pub paddle_bounce_combo: usize,

    // Number of blocks after one paddle bounce
    pub single_bounce_combo: usize
}


impl MatchState {
    pub fn reset(&mut self) {
        self.blocks = 0;
        self.paddle_bounces = 0;
        self.points = 0;
        self.wall_bounces = 0;
        self.direct_hit_possible = false;
        self.paddle_bounce_combo = 0;
        self.single_bounce_combo = 0;
        self.direct_hits = 0;
    }
}

impl Default for MatchState {
    fn default() -> Self {
        MatchState {
            blocks: 0,
            points: 0,
            paddle_bounces: 0,
            wall_bounces: 0,
            direct_hit_possible: false,
            direct_hits: 0,
            paddle_bounce_combo: 0,
            single_bounce_combo: 0,
        }
    }
}


impl MatchState {

    pub fn add_paddle_bounce(&mut self) {
        self.paddle_bounces += 1;
        self.direct_hit_possible = true;
        self.single_bounce_combo = 0;
    }

    pub fn ball_launched(&mut self) {
        self.direct_hit_possible = false;
    }

    // Only when ball removed
    pub fn add_block_hit(&mut self) {
        self.blocks -= 1;
        self.single_bounce_combo += 1;

        if self.direct_hit_possible {
            self.direct_hits += 1;

        }
    }

    pub fn add_wall_hit(&mut self) {
        self.wall_bounces += 1;
        self.direct_hit_possible = false;
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
        s.add_block_hit();
        s.add_paddle_bounce();
        s.add_wall_hit();
        s.add_block_hit();
        s.add_paddle_bounce();
        s.add_block_hit();

        assert_eq!(s.direct_hits, 2);
        assert_eq!(s.blocks, 97);
        assert_eq!(s.wall_bounces, 1);
        assert_eq!(s.paddle_bounces, 3);
    }


    // Number of blocks after one paddle bounce
    #[test]
    fn handles_single_bounce_combo() {
        let mut s = MatchState::default();
        s.blocks = 100;

        s.add_paddle_bounce();
        s.add_block_hit();
        s.add_block_hit();
        s.add_block_hit();

        assert_eq!(s.single_bounce_combo, 3);

        s.add_wall_hit();
        assert_eq!(s.single_bounce_combo, 3);

        s.add_paddle_bounce();
        assert_eq!(s.single_bounce_combo, 0);
    }




}