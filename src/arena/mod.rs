use bevy::prelude::{Commands, Transform, TransformBundle};
use bevy_rapier2d::geometry::{Collider, Friction, Restitution};
use bevy_rapier2d::math::Real;
use crate::config::{ARENA_HEIGHT_H, ARENA_WIDTH_H, MAX_RESTITUTION, SCREEN_HEIGHT, SCREEN_HEIGHT_H, SCREEN_WIDTH, SCREEN_WIDTH_H};

pub fn spawn_arena(mut commands: Commands) {
    commands
        .spawn(Collider::cuboid(ARENA_WIDTH_H, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -ARENA_HEIGHT_H, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(ARENA_WIDTH_H, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, ARENA_HEIGHT_H, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, ARENA_HEIGHT_H))
        .insert(TransformBundle::from(Transform::from_xyz(-ARENA_WIDTH_H, 0.0, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, ARENA_HEIGHT_H))
        .insert(TransformBundle::from(Transform::from_xyz(ARENA_WIDTH_H, 0.0, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));
}
