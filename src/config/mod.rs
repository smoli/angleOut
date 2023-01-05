use bevy_rapier2d::math::Real;

pub const PIXELS_PER_METER: f32 = 100.0;

pub const SCREEN_WIDTH: Real = 1000.0;
pub const SCREEN_HEIGHT: Real = 500.0;
pub const SCREEN_WIDTH_H: Real = SCREEN_WIDTH / 2.0;
pub const SCREEN_HEIGHT_H: Real = 500.0 / 2.0;
pub const BALL_SIZE: Real = 10.0;
pub const MAX_BALL_SPEED: Real = 500.0;
pub const MAX_RESTITUTION: Real = 1.0;

pub const PADDLE_WIDTH: Real = 150.0;
pub const PADDLE_WIDTH_H: Real = PADDLE_WIDTH / 2.0;
pub const PADDLE_THICKNESS: Real = 10.0;
pub const PADDLE_LIFT: Real = PADDLE_THICKNESS * 3.0;

pub const PADDLE_ROTATION_ACCEL:Real = 5.0;
pub const PADDLE_POSITION_ACCEL:Real = 5.0;

pub const PADDLE_RESTING_Y: Real = -SCREEN_HEIGHT_H + PADDLE_LIFT;
pub const PADDLE_RESTING_X: Real = 0.0;
pub const PADDLE_RESTING_ROTATION: Real = 0.0;

pub const BLOCK_WIDTH:Real = 50.0;
pub const BLOCK_WIDTH_H:Real = BLOCK_WIDTH / 2.0;
pub const BLOCK_HEIGHT:Real = 15.0;
pub const BLOCK_HEIGHT_H:Real = BLOCK_HEIGHT / 2.0;

const ARENA_WIDTH:Real = SCREEN_WIDTH / 2.0;
pub const ARENA_WIDTH_H:Real = ARENA_WIDTH / 2.0;
const ARENA_HEIGHT:Real = SCREEN_HEIGHT;
pub const ARENA_HEIGHT_H:Real = ARENA_HEIGHT / 2.0;
