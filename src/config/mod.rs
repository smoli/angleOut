use bevy_rapier3d::geometry::Group;

pub const SCREEN_WIDTH: f32 = 1600.0;
pub const SCREEN_HEIGHT: f32 = 900.0;
pub const BALL_RADIUS: f32 = 0.35 / 2.0;
pub const MAX_BALL_SPEED: f32 = 20.0;
pub const MIN_BALL_SPEED: f32 = 12.0;
pub const MAX_RESTITUTION: f32 = 1.0;

pub const PADDLE_WIDTH: f32 = 2.0;
pub const PADDLE_WIDTH_H: f32 = PADDLE_WIDTH / 2.0;
pub const PADDLE_THICKNESS: f32 = 1.05;
pub const PADDLE_LIFT: f32 = PADDLE_THICKNESS * 1.0;

pub const PADDLE_ROTATION_ACCEL:f32 = 5.0;
pub const PADDLE_POSITION_ACCEL:f32 = 3.0;
pub const PADDLE_RESTING_Z: f32 = 7.0;
pub const PADDLE_RESTING_Y: f32 = 0.0;
pub const PADDLE_RESTING_X: f32 = 0.0;
pub const PADDLE_RESTING_ROTATION: f32 = 0.0;

pub const ARENA_WIDTH:f32 = 20.0;
pub const ARENA_WIDTH_H:f32 = ARENA_WIDTH / 2.0;
pub const ARENA_HEIGHT:f32 = 14.0;
pub const ARENA_HEIGHT_H:f32 = ARENA_HEIGHT / 2.0;

pub const PADDLE_BOUNCE_IMPULSE: f32 = 5.0;
pub const PADDLE_LAUNCH_IMPULSE: f32 = 10.0;

pub const COLLIDER_GROUP_NONE:Group   = Group::empty();
pub const COLLIDER_GROUP_BALL:Group   = Group::GROUP_1;
pub const COLLIDER_GROUP_PADDLE:Group = Group::GROUP_2;
pub const COLLIDER_GROUP_BLOCK:Group  = Group::GROUP_3;
pub const COLLIDER_GROUP_ARENA:Group  = Group::GROUP_4;
pub const COLLIDER_GROUP_DEATH:Group  = Group::GROUP_5;
