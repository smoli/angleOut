use bevy::prelude::{Commands, Transform, TransformBundle};
use bevy_rapier2d::geometry::{Collider, Friction, Restitution};
use bevy_rapier2d::math::Real;
use crate::config::{MAX_RESTITUTION, SCREEN_HEIGHT_H, SCREEN_WIDTH_H};

pub fn spawn_arena(mut commands: Commands) {
    commands
        .spawn(Collider::cuboid(SCREEN_WIDTH_H, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -SCREEN_HEIGHT_H, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(SCREEN_WIDTH_H, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, SCREEN_HEIGHT_H, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, SCREEN_HEIGHT_H))
        .insert(TransformBundle::from(Transform::from_xyz(-SCREEN_WIDTH_H, 0.0, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, SCREEN_HEIGHT_H))
        .insert(TransformBundle::from(Transform::from_xyz(SCREEN_WIDTH_H, 0.0, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));
}
