use bevy_rapier3d::geometry::Group;

pub const SCREEN_WIDTH: f32 = 1600.0;
pub const SCREEN_HEIGHT: f32 = 800.0;
pub const BALL_RADIUS: f32 = 3.5 / 2.0;
pub const MAX_BALL_SPEED: f32 = 130.0;
pub const MIN_BALL_SPEED: f32 = 130.0;
pub const MAX_RESTITUTION: f32 = 1.0;


// This is basically a factor that the dispatcher vector between the ship an the ball is multiplied.
pub const GRAB_FORCE_MAGNITUDE: f32 = 300.0;

// Max Distance where grab force will be applied
pub const GRAB_ATTRACT_RADIUS: f32 = 30.0;

// Distance at which the ball is actually grabbed and will be made inactive
pub const GRAB_RADIUS: f32 = 2.0;

pub const PADDLE_WIDTH: f32 = 20.0;
pub const PADDLE_WIDTH_H: f32 = PADDLE_WIDTH / 2.0;
pub const PADDLE_THICKNESS: f32 = 10.5;
pub const PADDLE_LIFT: f32 = PADDLE_THICKNESS * 3.0;

pub const PADDLE_ROTATION_ACCEL:f32 = 5.0;
pub const PADDLE_POSITION_ACCEL_ACCEL: f32 = 3.0;
pub const PADDLE_POSITION_MAX_ACCEL:f32 = 2.5;
pub const PADDLE_RESTING_Z: f32 = 70.0;
pub const PADDLE_RESTING_Y: f32 = 0.0;
pub const PADDLE_RESTING_X: f32 = 0.0;
pub const PADDLE_RESTING_ROTATION: f32 = 0.0;

pub const ARENA_WIDTH:f32 = 200.0;
pub const ARENA_WIDTH_H:f32 = ARENA_WIDTH / 2.0;
pub const ARENA_HEIGHT:f32 = 140.0;
pub const ARENA_HEIGHT_H:f32 = ARENA_HEIGHT / 2.0;
pub const BACKGROUND_SPEED: f32 = 20.0;
pub const BACKGROUND_LENGTH: f32 = 400.0;

pub const PADDLE_BOUNCE_IMPULSE: f32 = 50.0;
pub const PADDLE_LAUNCH_IMPULSE: f32 = 200.0;

pub const COLLIDER_GROUP_NONE:Group   = Group::empty();
pub const COLLIDER_GROUP_BALL:Group   = Group::GROUP_1;
pub const COLLIDER_GROUP_PADDLE:Group = Group::GROUP_2;
pub const COLLIDER_GROUP_BLOCK:Group  = Group::GROUP_3;
pub const COLLIDER_GROUP_ARENA:Group  = Group::GROUP_4;
pub const COLLIDER_GROUP_DEATH:Group  = Group::GROUP_5;

pub const BLOCK_WIDTH: f32 = 15.0;
pub const BLOCK_WIDTH_H: f32 = BLOCK_WIDTH / 2.0;
pub const BLOCK_HEIGHT: f32 = 3.75;
pub const BLOCK_DEPTH: f32 = 7.51;
pub const BLOCK_GAP: f32 = 2.0;
pub const BLOCK_ROUNDNESS: f32 = 0.2;

