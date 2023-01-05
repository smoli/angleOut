use bevy::math::Vec3;
use bevy::prelude::Resource;
use bevy_rapier2d::prelude::Real;

#[derive(Resource)]
pub struct GameState {
    pub paddle_rotation: Real,
    pub paddle_position: Vec3
}