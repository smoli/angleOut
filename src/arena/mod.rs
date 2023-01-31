use bevy::pbr::NotShadowReceiver;
use bevy::prelude::{Component, App, AssetServer, Commands, default, Plugin, Res, SystemSet, TransformBundle, Transform, Query, With, Time, IntoSystemDescriptor, Entity, DespawnRecursiveExt, Assets, ResMut, MaterialPlugin, MaterialMeshBundle, shape, Mesh};
use bevy_rapier3d::dynamics::CoefficientCombineRule;
use bevy_rapier3d::prelude::{Collider, Friction, Restitution, RigidBody};
use crate::config::{ARENA_HEIGHT_H, ARENA_WIDTH_H, BACKGROUND_LENGTH, BACKGROUND_SPEED, MAX_RESTITUTION};
use crate::labels::SystemLabels;
use crate::materials::arena::ArenaMaterial;
use crate::materials::background::BackgroundMaterial;
use crate::physics::{Collidable, CollidableKind};
use crate::state::GameState;

#[derive(Component)]
pub struct Arena;

#[derive(Component)]
pub struct Scrollable {
    speed: f32,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_plugin(
                MaterialPlugin::<ArenaMaterial>::default(),
            )

            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(arena_spawn)
            )
            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(arena_scroll.label(SystemLabels::UpdateWorld))
                    .with_system(arena_update_background_material.label(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatch)
                    .with_system(arena_despawn)
            )
        ;
    }
}

fn arena_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArenaMaterial>>,
) {
   /* commands
        .spawn(SceneBundle {
            scene: asset_server.load("ship3_003.glb#Scene2"),
            ..default()
        })
        .insert(
            Arena
        )
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -4.0, 0.0)))
        .insert(Scrollable {
            speed: BACKGROUND_SPEED,
        });

    commands
        .spawn(SceneBundle {
            scene: asset_server.load("ship3_003.glb#Scene2"),
            ..default()
        })
        .insert(
            Arena
        )
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -4.0, -BACKGROUND_LENGTH)))
        .insert(Scrollable {
            speed: BACKGROUND_SPEED,
        })
    ;
*/

    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Plane{ size: ARENA_WIDTH_H * 2.0 })),
            material: materials.add(ArenaMaterial {
                color1: Default::default(),
                color2: Default::default(),
                time: 0.0,
                alpha_mode: Default::default(),
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(NotShadowReceiver)
        .insert(Arena);

    let wall_thickness = 10.0;
    // Left
    commands.spawn(Collider::cuboid(wall_thickness, 60.0, 200.0))
        .insert(TransformBundle::from(Transform::from_xyz(-ARENA_WIDTH_H - wall_thickness, 0.0, 0.0)))
        .insert(Restitution {
            coefficient: MAX_RESTITUTION,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Friction::coefficient(0.0))
        .insert(Collidable {
            kind: CollidableKind::Wall,
        })
        .insert(
            Arena
        )
    ;

    // Right
    commands.spawn(Collider::cuboid(wall_thickness, 60.0, 200.0))
        .insert(TransformBundle::from(Transform::from_xyz(ARENA_WIDTH_H + wall_thickness, 0.0, 0.0)))
        .insert(Restitution {
            coefficient: MAX_RESTITUTION,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Friction::coefficient(0.0))
        .insert(Collidable {
            kind: CollidableKind::Wall,
        })
        .insert(
            Arena
        )
    ;

    // Top
    commands
        .spawn(RigidBody::Fixed)
        .insert(Collider::cuboid(ARENA_WIDTH_H, 60.0, wall_thickness))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, -ARENA_HEIGHT_H - 13.0 - wall_thickness)))
        .insert(Restitution {
            coefficient: MAX_RESTITUTION,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Friction::coefficient(0.0))
        .insert(Collidable {
            kind: CollidableKind::Wall,
        })
        .insert(
            Arena
        )
    ;

    // Bottom
    commands.spawn(Collider::cuboid(ARENA_WIDTH_H, 60.0, wall_thickness))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, ARENA_HEIGHT_H + 50.0 + wall_thickness)))
        // .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(Collidable {
            kind: CollidableKind::DeathTrigger,
        })
        .insert(
            Arena
        )
    ;
}

fn arena_despawn(
    mut commands: Commands,
    arena_parts: Query<Entity, With<Arena>>
) {
    for part in &arena_parts {
        commands.entity(part)
            .despawn_recursive();
    }
}

fn arena_scroll(
    time: Res<Time>,
    mut scrollables: Query<(&mut Transform, &Scrollable)>) {
    for (mut trans, scrollable) in &mut scrollables {
        trans.translation.z += scrollable.speed * time.delta_seconds();

        if trans.translation.z > BACKGROUND_LENGTH {
            trans.translation.z -= 2.0 * BACKGROUND_LENGTH;
        }
    }
}

fn arena_update_background_material(
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    time: Res<Time>,
) {
    for (_, mut mat) in materials.iter_mut() {
        mat.time = time.elapsed_seconds();
    }
}