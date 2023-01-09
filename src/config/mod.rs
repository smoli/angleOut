use std::convert::Into;
use bevy_rapier3d::geometry::Group;
use bevy_rapier3d::math::Real;

pub const PIXELS_PER_METER: f32 = 100.0;

pub const SCREEN_WIDTH: Real = 1600.0;
pub const SCREEN_HEIGHT: Real = 900.0;
pub const SCREEN_WIDTH_H: Real = SCREEN_WIDTH / 2.0;
pub const SCREEN_HEIGHT_H: Real = SCREEN_HEIGHT / 2.0;
pub const BALL_RADIUS: Real = 0.35 / 2.0;
pub const MAX_BALL_SPEED: Real = 30.0;
pub const MIN_BALL_SPEED: Real = 12.0;
pub const MAX_RESTITUTION: Real = 1.0;

pub const PADDLE_WIDTH: Real = 2.0;
pub const PADDLE_WIDTH_H: Real = PADDLE_WIDTH / 2.0;
pub const PADDLE_THICKNESS: Real = 1.05;
pub const PADDLE_THICKNESS_H: Real = PADDLE_THICKNESS / 2.0;
pub const PADDLE_LIFT: Real = PADDLE_THICKNESS * 1.0;

pub const PADDLE_ROTATION_ACCEL:Real = 5.0;
pub const PADDLE_POSITION_ACCEL:Real = 3.0;

pub const PADDLE_RESTING_Z: Real = 7.0;
pub const PADDLE_RESTING_Y: Real = 0.0;
pub const PADDLE_RESTING_X: Real = 0.0;
pub const PADDLE_RESTING_ROTATION: Real = 0.0;

pub const BLOCK_WIDTH:Real = 50.0;
pub const BLOCK_WIDTH_H:Real = BLOCK_WIDTH / 2.0;
pub const BLOCK_HEIGHT:Real = 15.0;
pub const BLOCK_HEIGHT_H:Real = BLOCK_HEIGHT / 2.0;

pub const ARENA_WIDTH:Real = 20.0;
pub const ARENA_WIDTH_H:Real = ARENA_WIDTH / 2.0;
pub const ARENA_HEIGHT:Real = 14.0;
pub const ARENA_HEIGHT_H:Real = ARENA_HEIGHT / 2.0;

pub const COLLIDER_GROUP_NONE:Group   = Group::empty();
pub const COLLIDER_GROUP_BALL:Group   = Group::GROUP_1;
pub const COLLIDER_GROUP_PADDLE:Group = Group::GROUP_2;
pub const COLLIDER_GROUP_BLOCK:Group  = Group::GROUP_3;
pub const COLLIDER_GROUP_ARENA:Group  = Group::GROUP_4;
pub const COLLIDER_GROUP_DEATH:Group  = Group::GROUP_5;


pub const BLOCK_GAP: Real = BLOCK_WIDTH / 5.0;