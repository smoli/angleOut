use bevy::math::Vec3;
use bevy::prelude::Resource;
use bevy_rapier2d::prelude::Real;

#[derive(Resource)]
pub struct PaddleState {
    pub paddle_rotation: Real,
    pub paddle_position: Vec3
}