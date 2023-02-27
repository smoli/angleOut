use std::f32::consts::PI;
use bevy::hierarchy::{BuildChildren, Parent};
use bevy::log::info;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::pbr::{NotShadowReceiver, StandardMaterial};
use bevy::prelude::{Component, App, AssetServer, Commands, default, Plugin, Res, SystemSet, TransformBundle, Transform, Query, With, Time, IntoSystemDescriptor, Entity, DespawnRecursiveExt, Assets, ResMut, MaterialPlugin, MaterialMeshBundle, shape, Mesh, Color, AlphaMode, SceneBundle, Handle, Without, Name};
use bevy_rapier3d::dynamics::CoefficientCombineRule;
use bevy_rapier3d::na::inf;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionEvent, CollisionGroups, Friction, Restitution, RigidBody, Sensor};

use crate::config::{ARENA_HEIGHT, ARENA_HEIGHT_H, ARENA_WIDTH, ARENA_WIDTH_H, BACKGROUND_LENGTH, BACKGROUND_SPEED, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_DEATH, MAX_RESTITUTION};
use crate::labels::SystemLabels;
use crate::level::{LevelObstacle, Levels};
use crate::materials::arena::ArenaMaterial;
use crate::materials::background::BackgroundMaterial;
use crate::materials::CustomMaterialApplied;
use crate::materials::force_field::ForceFieldMaterial;
use crate::physics::{Collidable, CollidableKind, COLLISION_EVENT_HANDLING, CollisionInfo, CollisionTag};
use crate::state::GameState;

#[derive(Component)]
pub struct Arena;

#[derive(Component)]
pub struct ForceField;


#[derive(Component)]
pub struct Scrollable {
    speed: f32,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(
                MaterialPlugin::<ForceFieldMaterial>::default(),
            )

            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(arena_spawn)
            )
            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(arena_scroll.label(SystemLabels::UpdateWorld))
                    .with_system(arena_update_force_field_material.label(SystemLabels::UpdateWorld))
            )


            .add_system_to_stage(COLLISION_EVENT_HANDLING, arena_handle_collisions)

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatch)
                    .with_system(arena_despawn)
            )
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
        .spawn(SceneBundle {
            scene: asset_server.load(level.background_asset.clone()),
            ..default()
        })
        .insert(
            Arena
        )
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -4.0, 0.0)))
        .insert(Scrollable {
            speed: level.background_scroll_velocity.clone(),
        })
    ;

    commands
        .spawn(SceneBundle {
            scene: asset_server.load(level.background_asset.clone()),
            ..default()
        })
        .insert(
            Arena
        )
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -4.0, -BACKGROUND_LENGTH)))
        .insert(Scrollable {
            speed: level.background_scroll_velocity,
        })
    ;

    commands
        .spawn(SceneBundle {
            scene: asset_server.load(level.background_asset.clone()),
            ..default()
        })
        .insert(
            Arena
        )
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -4.0, - 2.0 * BACKGROUND_LENGTH)))
        .insert(Scrollable {
            speed: level.background_scroll_velocity,
        })
    ;


    let wall_thickness = 100.0;
    // Left
    if level.default_wall_l {
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
    }

    // Right
    if level.default_wall_r {
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
    }


    // Top Barrier
    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Quad {
                size: Vec2::new(ARENA_WIDTH, 20.0),
                flip: false,
            })),
            material: force_field_mat.add(ForceFieldMaterial {
                color1: Color::BLUE,
                color_texture: Some(asset_server.load("hexagon2.png")),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, -ARENA_HEIGHT_H - 13.0),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),

        })

        .insert(ForceField)
        .with_children(|parent| {
            parent
                .spawn(RigidBody::Fixed)
                .insert(Collider::cuboid(ARENA_WIDTH_H, 60.0, wall_thickness))
                .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, -ARENA_HEIGHT_H - 25.0)))
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
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, ARENA_HEIGHT_H + 50.0 + wall_thickness)))
        // .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
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

                commands
                    .spawn(MaterialMeshBundle {
                        mesh: meshes.add(Mesh::from(shape::Quad {
                            size: Vec2::new(*size, 20.0),
                            flip: false,
                        })),
                        material: force_field_mat.add(ForceFieldMaterial {
                            color1: Color::BLUE,
                            color_texture: Some(asset_server.load("hexagon2.png")),
                            ..default()
                        }),
                        transform: Transform::from_translation(origin.clone()).with_rotation(Quat::from_rotation_y(angle)),
                        global_transform: Default::default(),
                        visibility: Default::default(),
                        computed_visibility: Default::default(),

                    })

                    .insert(ForceField)
                    .insert(Arena)
                    .with_children(|parent| {
                        parent
                            .spawn(RigidBody::Fixed)
                            .insert(Collider::cuboid(size / 2.0, size / 2.0, size / 2.0))
                            .insert(TransformBundle::from(Transform::from_translation(-*size * 0.5 * collider_move_vec)))
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
                    .insert(TransformBundle::from(Transform::from_translation(pos.clone())))
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
                    .insert(TransformBundle::from(Transform::from_translation(origin.clone()).with_rotation(rot)))
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
            .despawn_recursive();
    }
}

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
            .remove::<Handle<StandardMaterial>>()
            .insert(materials.add(ArenaMaterial {
                color1: Default::default(),
                color2: Default::default(),
                time: 0.0,
                alpha_mode: AlphaMode::Blend,
            }))
        ;
    }
}

fn arena_scroll(
    time: Res<Time>,
    mut scrollables: Query<(&mut Transform, &Scrollable)>) {
    for (mut trans, scrollable) in &mut scrollables {
        trans.translation.z += scrollable.speed * time.delta_seconds();

        if trans.translation.z > BACKGROUND_LENGTH {
            trans.translation.z -= 3.0 * BACKGROUND_LENGTH;
        }
    }
}

fn arena_update_force_field_material(
    mut materials: ResMut<Assets<ForceFieldMaterial>>,
    time: Res<Time>,
) {
    for (_, mut mat) in materials.iter_mut() {
        mat.time = time.elapsed_seconds();
    }
}

fn arena_handle_collisions(
    mut commands: Commands,
    mut wall: Query<(Entity, &Parent), (With<Arena>, With<CollisionTag>)>,
    forceFields: Query<(&ForceField, &Handle<ForceFieldMaterial>)>,
    collisions: Res<CollisionInfo>,
    mut materials: ResMut<Assets<ForceFieldMaterial>>,
    time: Res<Time>,
) {
    for (wall, parent) in &wall {
        if let Some(collisions) = collisions.collisions.get(&wall) {
            for collision in collisions {
                match collision.other {
                    CollidableKind::Ball => {
                        let p = forceFields.get(parent.get());

                        if let Ok(forceField) = p {
                            if let Some(mat) = materials.get_mut(forceField.1) {
                                mat.hit_time = time.elapsed_seconds();
                                let h_x = (collision.other_pos.x + ARENA_WIDTH_H) / ARENA_WIDTH;
                                mat.hit_position = Vec3::new(h_x, 0.0, 0.0);
                                //info!("You hit that wall at {}!", h_x);
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}