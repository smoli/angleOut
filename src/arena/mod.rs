use bevy::app::{App, Plugin, PostUpdate, Update};
use bevy::math::{Quat, Vec2, Vec3};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{in_state, AlphaMode, Assets, AssetServer, ChildOf, Commands, Component, Entity, GlobalTransform, IntoScheduleConfigs, MaterialPlugin, Mesh, Mesh3d, Name, OnEnter, OnExit, Query, Rectangle, Res, ResMut, Time, Transform, With, Without};
use bevy::world_serialization::WorldAssetRoot;
use bevy_rapier3d::dynamics::CoefficientCombineRule;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionGroups, Friction, Restitution, RigidBody, Sensor};

use crate::config::{ARENA_HEIGHT_H, ARENA_WIDTH, ARENA_WIDTH_H, BACKGROUND_LENGTH, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_DEATH, MAX_RESTITUTION};
use crate::labels::SystemLabels;
use crate::level::{LevelObstacle, Levels};
use crate::materials::arena::ArenaMaterial;
use crate::materials::CustomMaterialApplied;
use crate::materials::force_field::{panel_uv, ForceFieldMaterial};
use crate::physics::{Collidable, CollidableKind, CollisionEventHandling, CollisionInfo, CollisionTag};
use crate::state::GameState;

#[derive(Component)]
pub struct Arena;

/// A force field panel. Carries its own extents so an impact can be mapped
/// into the panel's uv space rather than approximated from the arena width.
#[derive(Component)]
pub struct ForceField {
    pub size: Vec2,
}


#[derive(Component)]
pub struct Scrollable {
    speed: f32,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                MaterialPlugin::<ForceFieldMaterial>::default(),
            )

            .add_systems(OnEnter(GameState::InMatch), arena_spawn)
            .add_systems(
                Update,
                arena_scroll
                    .in_set(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::InMatch)),
            )


            .add_systems(PostUpdate, arena_handle_collisions.in_set(CollisionEventHandling))

            .add_systems(OnExit(GameState::PostMatch), arena_despawn)
        ;
    }
}

fn arena_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    levels: Res<Levels>,
    mut force_field_mat: ResMut<Assets<ForceFieldMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let level = levels.get_current_level().unwrap();

    commands
        .spawn(WorldAssetRoot(asset_server.load(level.background_asset.clone())))
        .insert(
            Arena
        )
        .insert(Transform::from_xyz(0.0, -4.0, 0.0))
        .insert(Scrollable {
            speed: level.background_scroll_velocity.clone(),
        })
    ;

    commands
        .spawn(WorldAssetRoot(asset_server.load(level.background_asset.clone())))
        .insert(
            Arena
        )
        .insert(Transform::from_xyz(0.0, -4.0, -BACKGROUND_LENGTH))
        .insert(Scrollable {
            speed: level.background_scroll_velocity,
        })
    ;

    commands
        .spawn(WorldAssetRoot(asset_server.load(level.background_asset.clone())))
        .insert(
            Arena
        )
        .insert(Transform::from_xyz(0.0, -4.0, - 2.0 * BACKGROUND_LENGTH))
        .insert(Scrollable {
            speed: level.background_scroll_velocity,
        })
    ;


    let wall_thickness = 100.0;
    // Left
    if level.default_wall_l {
        commands.spawn(Collider::cuboid(wall_thickness, 60.0, 200.0))
            .insert(Transform::from_xyz(-ARENA_WIDTH_H - wall_thickness, 0.0, 0.0))
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
    }

    // Right
    if level.default_wall_r {
        commands.spawn(Collider::cuboid(wall_thickness, 60.0, 200.0))
            .insert(Transform::from_xyz(ARENA_WIDTH_H + wall_thickness, 0.0, 0.0))
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
    }


    // Top Barrier
    let top_barrier_size = Vec2::new(ARENA_WIDTH, 20.0);
    commands
        .spawn((
            Mesh3d(meshes.add(Mesh::from(Rectangle::from_size(top_barrier_size)))),
            MeshMaterial3d(force_field_mat.add(ForceFieldMaterial::for_panel(
                top_barrier_size,
                asset_server.load("hexagon2.png"),
            ))),
            Transform::from_xyz(0.0, 0.0, -ARENA_HEIGHT_H - 13.0),
        ))

        .insert(ForceField { size: top_barrier_size })
        .with_children(|parent| {
            parent
                .spawn(RigidBody::Fixed)
                .insert(Collider::cuboid(ARENA_WIDTH_H, 60.0, wall_thickness))
                .insert(Transform::from_xyz(0.0, 0.0, -ARENA_HEIGHT_H - 25.0))
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
                );
        });

    // Bottom
    commands.spawn(Collider::cuboid(ARENA_WIDTH_H, 60.0, wall_thickness))
        .insert(Transform::from_xyz(0.0, 0.0, ARENA_HEIGHT_H + 50.0 + wall_thickness))
        // .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(Collidable {
            kind: CollidableKind::DeathTrigger,
        })
        .insert(
            Arena
        )
    ;


    for o in &level.obstacles {
        match o {
            LevelObstacle::ForceField(origin, normal, size, flip) => {
                let angle = Vec3::Z.angle_between(*normal) * if *flip { -1.0 } else { 1.0 };
                let rot = Quat::from_rotation_y(-angle);
                let collider_move_vec = rot * *normal;

                let panel_size = Vec2::new(*size, 20.0);

                commands
                    .spawn((
                        Mesh3d(meshes.add(Mesh::from(Rectangle::from_size(panel_size)))),
                        MeshMaterial3d(force_field_mat.add(ForceFieldMaterial::for_panel(
                            panel_size,
                            asset_server.load("hexagon2.png"),
                        ))),
                        Transform::from_translation(origin.clone()).with_rotation(Quat::from_rotation_y(angle)),
                    ))

                    .insert(ForceField { size: panel_size })
                    .insert(Arena)
                    .with_children(|parent| {
                        parent
                            .spawn(RigidBody::Fixed)
                            .insert(Collider::cuboid(size / 2.0, size / 2.0, size / 2.0))
                            .insert(Transform::from_translation(-*size * 0.5 * collider_move_vec))
                            .insert(Restitution {
                                coefficient: MAX_RESTITUTION,
                                combine_rule: CoefficientCombineRule::Max,
                            })
                            .insert(Friction::coefficient(0.0))
                            .insert(Collidable {
                                kind: CollidableKind::Wall,
                            })
                            .insert(CollisionGroups::new(COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_BALL))

                            .insert(
                                Arena
                            );
                    });
            }

            LevelObstacle::Box(pos, w, h) => {
                commands
                    .spawn(RigidBody::Fixed)
                    .insert(Collider::cuboid(w / 2.0, 100.0, h / 2.0))
                    .insert(Transform::from_translation(pos.clone()))
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
                    );
            }

            LevelObstacle::DirectionalDeathTrigger(origin, normal, size) => {
                let angle = Vec3::Z.angle_between(*normal);
                let rot = Quat::from_rotation_y(angle);


                commands
                    .spawn(RigidBody::Fixed)
                    .insert(Collider::cuboid(size / 2.0, size / 2.0, size / 2.0))
                    .insert(Transform::from_translation(origin.clone()).with_rotation(rot))
                    .insert(Collidable {
                        kind: CollidableKind::DirectionalDeathTrigger(normal.clone()),
                    })
                    .insert(CollisionGroups::new(COLLIDER_GROUP_DEATH, COLLIDER_GROUP_BLOCK))
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(Sensor)

                ;
            }
        };
    }
}

fn arena_despawn(
    mut commands: Commands,
    arena_parts: Query<Entity, With<Arena>>,
) {
    for part in &arena_parts {
        //info!("Despawn arena");
        commands.entity(part)
            .despawn();
    }
}

#[allow(dead_code)]
fn arena_set_custom_material(
    mut commands: Commands,
    arena: Query<(Entity, &Name), Without<CustomMaterialApplied>>,
    mut materials: ResMut<Assets<ArenaMaterial>>,
) {
    for (entity, name) in &arena {
        commands.entity(entity)
            .insert(CustomMaterialApplied);

        //info!("Applying Arena Material {}", name.as_ref());
        if name.as_ref() != "ValleyMesh" {
            continue;
        }

        commands.entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(materials.add(ArenaMaterial {
                color1: Default::default(),
                color2: Default::default(),
                time: 0.0,
                alpha_mode: AlphaMode::Blend,
            })))
        ;
    }
}

fn arena_scroll(
    time: Res<Time>,
    mut scrollables: Query<(&mut Transform, &Scrollable)>) {
    for (mut trans, scrollable) in &mut scrollables {
        trans.translation.z += scrollable.speed * time.delta_secs();

        if trans.translation.z > BACKGROUND_LENGTH {
            trans.translation.z -= 3.0 * BACKGROUND_LENGTH;
        }
    }
}

fn arena_handle_collisions(
    walls: Query<(Entity, &ChildOf), (With<Arena>, With<CollisionTag>)>,
    force_fields: Query<(&ForceField, &GlobalTransform, &MeshMaterial3d<ForceFieldMaterial>)>,
    collisions: Res<CollisionInfo>,
    mut materials: ResMut<Assets<ForceFieldMaterial>>,
    time: Res<Time>,
) {
    // The shader ages ripples against `globals.time`, which is the wrapped clock.
    let now = time.elapsed_secs_wrapped();

    for (wall, child_of) in &walls {
        let Some(collisions) = collisions.collisions.get(&wall) else {
            continue;
        };

        for collision in collisions {
            if collision.other != CollidableKind::Ball {
                continue;
            }

            let Ok((force_field, panel, material)) = force_fields.get(child_of.parent()) else {
                continue;
            };

            let Some(mut mat) = materials.get_mut(&material.0) else {
                continue;
            };

            mat.register_hit(panel_uv(panel, force_field.size, collision.other_pos), now);
        }
    }
}
