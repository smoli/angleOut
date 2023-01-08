use bevy::prelude::{AssetServer, Commands, Component, default, Res, SpriteBundle, Transform, TransformBundle, Vec3};
use bevy::prelude::Keyframes::Translation;
use bevy_rapier2d::geometry::{Collider, Friction, Restitution};
use bevy_rapier2d::math::Real;
use bevy_rapier2d::prelude::ActiveEvents;
use crate::config::{ARENA_HEIGHT, ARENA_HEIGHT_H, ARENA_WIDTH, ARENA_WIDTH_H, MAX_RESTITUTION, SCREEN_HEIGHT, SCREEN_HEIGHT_H, SCREEN_WIDTH, SCREEN_WIDTH_H};

#[derive(Component)]
pub struct LooseTrigger;

pub fn spawn_arena(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Collider::cuboid(10.0, 10.0))
        .insert(SpriteBundle {
            texture: asset_server.load("wall.png"),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -ARENA_HEIGHT_H, 0.0)))
        .insert(Transform {
            translation: Vec3::new(0.0, -ARENA_HEIGHT_H, 0.0),
            rotation: Default::default(),
            scale: Vec3::new(ARENA_WIDTH / 20.0, 1.0, 1.0),
        })
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(LooseTrigger)
        .insert(ActiveEvents::COLLISION_EVENTS);

    commands
        .spawn(Collider::cuboid(10.0, 10.0))
        .insert(SpriteBundle {
            texture: asset_server.load("wall.png"),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(0.0, ARENA_HEIGHT_H, 0.0)))
        .insert(Transform {
            translation: Vec3::new(0.0, ARENA_HEIGHT_H, 0.0),
            rotation: Default::default(),
            scale: Vec3::new(ARENA_WIDTH / 20.0, 1.0, 1.0),
        })
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, 10.0))
        .insert(SpriteBundle {
            texture: asset_server.load("wall.png"),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(-ARENA_WIDTH_H, 0.0, 0.0)))
        .insert(Transform {
            translation: Vec3::new(-ARENA_WIDTH_H, 0.0, 0.0),
            rotation: Default::default(),
            scale: Vec3::new(1.0, ARENA_HEIGHT / 20.0, 1.0),
        })
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, ARENA_HEIGHT_H))
        .insert(SpriteBundle {
            texture: asset_server.load("wall.png"),
            ..default()
        })
        .insert(TransformBundle::from(Transform::from_xyz(ARENA_WIDTH_H, 0.0, 0.0)))
        .insert(Transform {
            translation: Vec3::new(ARENA_WIDTH_H, 0.0, 0.0),
            rotation: Default::default(),
            scale: Vec3::new(1.0, ARENA_HEIGHT / 20.0, 1.0),
        })
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));
}
