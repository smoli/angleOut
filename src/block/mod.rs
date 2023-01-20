use std::f32::consts::TAU;
use std::time::Duration;
use bevy::utils::default;
use bevy::app::App;
use bevy::asset::{Asset, Assets, Handle};
use bevy::gltf::{Gltf, GltfMesh};
use bevy::log::{info};
use bevy::math::Vec2;
use bevy::pbr::{AlphaMode, MaterialMeshBundle, NotShadowCaster};
use bevy::prelude::{AssetServer, Color, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, IntoSystemDescriptor, Material, MaterialPlugin, Mesh, Name, PbrBundle, Plugin, Query, Res, ResMut, SceneBundle, StandardMaterial, SystemSet, Time, Transform, TransformBundle, Vec3, Visibility, With, Without};
use bevy::render::mesh::VertexAttributeValues;
use bevy::scene::Scene;
use bevy::time::FixedTimestep;
use bevy_rapier3d::prelude::{ActiveEvents, CoefficientCombineRule, Collider, CollisionGroups, ExternalForce, ExternalImpulse, Friction, LockedAxes, Restitution, RigidBody, Sensor};
use crate::ball::Ball;
use crate::config::{ARENA_WIDTH, ARENA_WIDTH_H, BALL_RADIUS, BLOCK_DEPTH, BLOCK_HEIGHT, BLOCK_ROUNDNESS, BLOCK_WIDTH, BLOCK_WIDTH_H, COLLIDER_GROUP_ARENA, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, MAX_RESTITUTION};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::materials::{CustomMaterial, CustomMaterialApplied};
use crate::MyAssetPack;
use crate::physics::{Collidable, CollidableKind, CollisionTag};
use crate::state::GameState;


#[derive(Debug, Clone)]
pub enum BlockType {
    Simple,
    Hardling,
    Concrete,
}

#[derive(Debug, Clone)]
pub enum BlockBehaviour {
    SittingDuck,
    Spinner,
    Vanisher,
    Repuslor,
    EvaderR,
    EvaderL,
    EvaderU,
    EvaderD,
}


#[derive(Component)]
struct BlockSpinner;

#[derive(Component)]
struct BlockVanisher;

#[derive(Component)]
struct BlockEvader {
    velocity: Vec3
}

#[derive(Component)]
struct BlockRepulsor;

#[derive(Component, Debug)]
pub struct Block {
    pub position: Vec2,
    pub asset_name: String,
    pub block_type: BlockType,
    pub behaviour: BlockBehaviour,
}

#[derive(Component, Debug)]
pub struct Hittable {
    pub hit_points: u8,
}


impl Default for Block {
    fn default() -> Self {
        Block {
            position: Default::default(),
            asset_name: "ship3_003.glb#Scene3".to_string(),
            block_type: BlockType::Simple,
            behaviour: BlockBehaviour::SittingDuck,
        }
    }
}


pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(
                MaterialPlugin::<CustomMaterial>::default(),
            )

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(block_spawn.label(SystemLabels::UpdateWorld))
                    .with_system(block_handle_collisions.label(SystemLabels::UpdateWorld))
                    .with_system(block_repluse.label(SystemLabels::UpdateWorld))
                    .with_system(block_update_evader)
                    .with_system(block_handle_evader_collisions)
                    // .with_system(block_custom_material)
                    // .with_system(block_update_custom_material)
            )

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(block_vanish)
                    .with_system(block_spin)
                    .with_run_criteria(FixedTimestep::step(1.0))
                    .label(SystemLabels::UpdateWorld)
            )

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatchLoose)
                    .with_system(block_despawn)
            )
        ;
    }
}


fn block_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    my: Res<MyAssetPack>,
    empties: Query<(Entity, &Block), With<RequestTag>>,
    assets_gltf: Res<Assets<Gltf>>
) {
    if let Some(gltf) = assets_gltf.get(&my.0) {
        for (entity, block) in &empties {
            let mut block_commands = commands.entity(entity);

            block_commands
                .remove::<RequestTag>()

                .insert(SceneBundle {
                    scene: gltf.named_scenes["003_SimpleBlock"].clone(),
                    ..default()
                })
                /*            .insert(PbrBundle {
                            mesh: asset_server.load(block.asset_name.as_str()),
                            ..default()
                        })*/
                .insert(TransformBundle::from(Transform::from_xyz(block.position.x, 0.0, block.position.y)))
                .insert(Collider::round_cuboid(
                    BLOCK_WIDTH / 2.0 - 2.0 * BLOCK_ROUNDNESS,
                    BLOCK_HEIGHT / 2.0 - 2.0 * BLOCK_ROUNDNESS,
                    BLOCK_DEPTH / 2.0 - 2.0 * BLOCK_ROUNDNESS,
                    BLOCK_ROUNDNESS,
                ))

                .insert(Restitution {
                    coefficient: MAX_RESTITUTION,
                    combine_rule: CoefficientCombineRule::Max,
                })
                .insert(Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                })
                .insert(CollisionGroups::new(COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_BALL | COLLIDER_GROUP_BLOCK | COLLIDER_GROUP_ARENA))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collidable {
                    kind: CollidableKind::Block
                })
            ;


            match block.behaviour {
                BlockBehaviour::Spinner => {
                    block_commands.insert(BlockSpinner);
                    block_commands.insert(RigidBody::Fixed);
                }

                BlockBehaviour::Vanisher => {
                    block_commands.insert(BlockVanisher);
                    block_commands.insert(RigidBody::Fixed);
                }

                BlockBehaviour::Repuslor => {
                    block_commands.insert(BlockRepulsor);
                    block_commands.insert(RigidBody::Fixed);
                }

                BlockBehaviour::EvaderR => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(50.0, 0.0, 0.0),
                    });
                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z));
                }

                BlockBehaviour::EvaderL => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(-50.0, 0.0, 0.0),
                    });
                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z));
                }

                BlockBehaviour::EvaderU => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(0.0, 0.0, -50.0),
                    });
                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_X));
                }

                BlockBehaviour::EvaderD => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(0.0, 0.0, 50.0),
                    });
                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_X));
                }

                _ => {}
            }

            match block.block_type {
                BlockType::Simple => {
                    block_commands.insert(Hittable {
                        hit_points: 1,
                    });
                }

                BlockType::Hardling => {
                    block_commands.insert(Hittable {
                        hit_points: 2,
                    });
                }

                BlockType::Concrete => {
                    block_commands.insert(Hittable {
                        hit_points: 3,
                    });
                }
            }
        }
    }
}


fn block_update_custom_material(
    mut materials: ResMut<Assets<CustomMaterial>>,
    time: Res<Time>
) {
    for (handle, mut mat) in materials.iter_mut() {
        mat.time = time.elapsed_seconds();
    }
}

fn block_custom_material(
    mut commands: Commands,
    blocks: Query<(Entity, &Handle<Mesh>, &Name), Without<CustomMaterialApplied>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>
) {
    for (block, handle, name) in &blocks {
        commands.entity(block)
            .insert(CustomMaterialApplied);

        if name.as_ref() != "SimpleBlock" {
            continue;
        }

       if let Some(mesh) = meshes.get_mut(handle) {
            if let Some(VertexAttributeValues::Float32x3(
                            positions,
                        )) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            {
                let colors: Vec<[f32; 4]> = positions
                    .iter()
                    .map(|[r, g, b]| {
                        [
                            (1. - *r) / 2.,
                            (1. - *g) / 2.,
                            (1. - *b) / 2.,
                            1.,
                        ]
                    })
                    .collect();
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    colors,
                );
            }
           let custom_material =
               custom_materials.add(CustomMaterial {
                   color1: Color::BLUE,
                   color2: Color::GOLD,
                   time: 0.0,
/*                   color_texture: None,*/
                   alpha_mode: AlphaMode::Blend,
               });


           commands
               .entity(block)
               .remove::<Handle<StandardMaterial>>();
           commands.entity(block).insert(custom_material);
/*            commands
                .entity(block)
                .remove::<Handle<StandardMaterial>>();*/


        }
    }
}

fn block_despawn(
    mut commands: Commands,
    blocks: Query<Entity, With<Block>>,
) {
    for block in &blocks {
        commands.entity(block)
            .despawn_recursive();
    }
}

fn block_handle_collisions(
    mut commands: Commands,
    mut blocks: Query<(Entity, &CollisionTag, &mut Hittable, &Block, &Transform)>,
    mut events: EventWriter<MatchEvent>,
    my: Res<MyAssetPack>,
    assets_gltf: Res<Assets<Gltf>>
) {
    for (entity, collision, mut hittable, block, trans) in &mut blocks {
        match collision.other {

            CollidableKind::Ball => {
                hittable.hit_points -= 1;

                if hittable.hit_points == 0 {
                    commands.entity(entity)
                        .despawn_recursive();
                    events.send(MatchEvent::TargetHit(collision.pos.clone(), block.block_type.clone(), block.behaviour.clone()));
                } else {
                    if let Some(gltf) = assets_gltf.get(&my.0) {

                    commands.entity(entity)
                        .remove::<SceneBundle>()
                        .insert(SceneBundle {
                            scene: gltf.named_scenes["006_Block_CrackedOnce"].clone(),
                            ..default()
                        })
                        .insert(TransformBundle::from_transform(trans.clone()));
                    }
                    info!("still alive")
                }
            }

            _ => {}
        }
    }
}

fn block_handle_evader_collisions(
    mut commands: Commands,
    mut blocks: Query<(Entity, &CollisionTag, &mut BlockEvader), With<Block>>,
    mut events: EventWriter<MatchEvent>,

) {
    for (block, collision, mut evader) in &mut blocks {
        match collision.other {

            CollidableKind::Block | CollidableKind::Wall => {
                evader.velocity *= -1.0;
            }

            CollidableKind::DeathTrigger => {
                commands.entity(block)
                    .despawn_recursive();
                events.send(MatchEvent::BlockLost);
            }

            _ => {}
        }
    }
}


fn block_spin(
    mut ball: Query<&mut Transform, With<BlockSpinner>>) {
    for mut trans in &mut ball {
        trans.rotate_y(0.25 * TAU);
    }
}

fn block_vanish(
    mut commands: Commands,
    mut blocks: Query<(Entity, &mut Visibility, &Transform), With<BlockVanisher>>,
    mut balls: Query<&Transform, With<Ball>>,
) {
    let mut positions: Vec<Vec3> = vec![];

    // Get all ball Positions - We need to check these to not make a block appear in a balls position
    // TODO: Is this loop/copy really better than iterating over &ball directly below?
    for t in &balls {
        positions.push(t.translation.clone());
    }

    let xd = BLOCK_DEPTH / 2.0 + BALL_RADIUS;
    let zd = BLOCK_WIDTH_H / 2.0 + BALL_RADIUS;


    for (block, mut vis, trans) in &mut blocks {
        let mut can_appear = true;


        for i in 0..positions.len() {
            let p = positions.get(i).unwrap();


            if (p.x - trans.translation.x).abs() < xd && (p.z - trans.translation.z).abs() < zd {
                can_appear = false;
                break;
            }
        }

        if !vis.is_visible {
            if can_appear {
                vis.is_visible = !vis.is_visible;
                commands.entity(block)
                    .remove::<Sensor>();
            }
        } else {
            vis.is_visible = !vis.is_visible;
            commands.entity(block)
                .insert(Sensor);
        }
    }
}

fn block_repluse(
    blocks: Query<(&Transform), With<BlockRepulsor>>,
    mut balls: Query<(&Transform, &mut ExternalForce), With<Ball>>,
) {
    let threshold = 20.0;
    let max_repulsion = 850.0;

    for (ball_pos, mut ball_force) in &mut balls {
        let mut bf = Vec3::ZERO;

        for block_trans in &blocks {
            let mut direction: Vec3 = block_trans.translation - ball_pos.translation;
            direction = direction.normalize();
            let dist = direction.length();

            if dist < threshold {
                let f = max_repulsion / threshold * dist;

                bf += direction * (f * -1.0)
            }
        }
        ball_force.force = bf;
    }
}


fn block_update_evader(
    time: Res<Time>,
    mut ball: Query<(&mut Transform, &mut BlockEvader)>
) {
    for (mut trans, mut evader) in &mut ball {
        trans.translation += evader.velocity * time.delta().as_secs_f32();
    }
}