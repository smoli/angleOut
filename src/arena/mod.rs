use bevy::prelude::{Component, App, AssetServer, Commands, default, info, Plugin, Res, SystemSet, TransformBundle, Transform};
use bevy::scene::SceneBundle;
use bevy_rapier3d::prelude::{Collider, Friction, Restitution, RigidBody};
use crate::config::{ARENA_HEIGHT_H, ARENA_WIDTH_H, MAX_RESTITUTION};
use crate::state::GameState;

#[derive(Component)]
pub struct Arena;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(arena_spawn)
            )
        ;
    }
}

fn arena_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("Loading ship!");

    commands
        .spawn(SceneBundle {
            scene: asset_server.load("ship3_003.gltf#Scene2"),
            ..default()
        })
        .insert(
            Arena
        )
    .insert(TransformBundle::from(Transform::from_xyz(0.0, -4.0, 0.0)));


    let wall_thickness = 10.0;
    // Left
    commands.spawn(Collider::cuboid(wall_thickness, 6.0, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(-ARENA_WIDTH_H - wall_thickness, 0.0, 0.0)))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))

    ;
    // Right
    commands.spawn(Collider::cuboid(wall_thickness, 6.0, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(ARENA_WIDTH_H + wall_thickness, 0.0, 0.0)))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))

    ;
    // Top
    commands
        .spawn(RigidBody::Fixed)
        .insert(Collider::cuboid(ARENA_WIDTH_H, 6.0, wall_thickness))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, -ARENA_HEIGHT_H - wall_thickness)))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))


    ;
    // Bottom
    commands.spawn(Collider::cuboid(ARENA_WIDTH_H, 6.0, wall_thickness))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, ARENA_HEIGHT_H + wall_thickness)))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))


    ;
}