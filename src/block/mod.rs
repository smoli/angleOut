pub mod trigger;

use std::f32::consts::TAU;
use std::time::Duration;

use bevy::app::App;
use bevy::asset::{Assets, AssetServer, Handle};
use bevy::gltf::{Gltf, GltfMesh};
use bevy::log::info;
use bevy::math::Vec2;
use bevy::pbr::MaterialMeshBundle;
use bevy::prelude::{Bundle, Color, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, IntoSystemDescriptor, MaterialPlugin, Plugin, Query, Res, ResMut, SystemSet, Time, Timer, TimerMode, Transform, TransformBundle, Vec3, Visibility, With, Without};
use bevy::prelude::KeyCode::C;
use bevy::time::FixedTimestep;
use bevy::utils::default;
use bevy::utils::hashbrown::HashMap;
use bevy_rapier3d::prelude::{ActiveEvents, CoefficientCombineRule, Collider, CollisionGroups, ExternalForce, Friction, LockedAxes, Restitution, RigidBody, Sensor};

use crate::ball::Ball;
use crate::block::trigger::{BlockTrigger, BlockTriggerTarget, BlockTriggerTargetInactive, TriggerGroup, TriggerStates, TriggerType};
use crate::config::{BALL_RADIUS, BLOCK_DEPTH, BLOCK_HEIGHT, BLOCK_ROUNDNESS, BLOCK_WIDTH_H, COLLIDER_GROUP_ARENA, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_DEATH, MAX_RESTITUTION};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::materials::block::BlockMaterial;
use crate::MyAssetPack;
use crate::physics::{Collidable, CollidableKind, COLLISION_EVENT_HANDLING, CollisionInfo, CollisionTag};
use crate::state::GameState;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Simple,
    Hardling,
    Concrete,
    Obstacle,
    SimpleTop,
}

#[derive(Debug, Clone)]
pub enum BlockBehaviour {
    SittingDuck,
    Spinner,
    Vanisher,
    Repuslor,
    EvaderR(f32),
    EvaderL(f32),
    EvaderU(f32),
    EvaderD(f32),
}


#[derive(Component)]
struct BlockSpinner;

#[derive(Component)]
struct BlockVanisher;

#[derive(Component)]
struct BlockEvader {
    velocity: Vec3,
}

#[derive(Component)]
struct BlockRepulsor;


#[derive(Component)]
struct Shaking {
    timer: Timer,
    original_position: Vec3,
    direction: Vec3,
}

#[derive(Component, Debug)]
pub struct Block {
    pub position: Vec2,
    pub asset_name: String,
    pub block_type: BlockType,
    pub behaviour: BlockBehaviour,
    pub material: Option<Handle<BlockMaterial>>,
    pub trigger_type: Option<TriggerType>,
    pub trigger_group: Option<TriggerGroup>,
}

#[derive(Component, Debug)]
pub struct Obstacle;

#[derive(Component, Debug)]
pub struct Hittable {
    pub hit_points: u8,
    pub original_hit_points: u8,
    pub only_top: bool,
}


impl Default for Hittable {
    fn default() -> Self {
        return Hittable {
            hit_points: 1,
            original_hit_points: 1,
            only_top: false,
        };
    }
}

impl Default for Block {
    fn default() -> Self {
        Block {
            position: Default::default(),
            asset_name: "003_SimpleBlock".to_string(),
            block_type: BlockType::Simple,
            behaviour: BlockBehaviour::SittingDuck,
            material: None,
            trigger_type: None,
            trigger_group: None,
        }
    }
}


pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(
                MaterialPlugin::<BlockMaterial>::default(),
            )

            .insert_resource(TriggerStates::new())

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(block_spawn.label(SystemLabels::UpdateWorld))
                    .with_system(block_repluse.label(SystemLabels::UpdateWorld))
                    .with_system(block_update_evader)
                    .with_system(block_shake.after(SystemLabels::UpdateWorld))
                    .with_system(block_update_custom_material)
                    .with_system(block_update_trigger_targets)
            )

            .add_system_to_stage(COLLISION_EVENT_HANDLING, block_handle_collisions)
            .add_system_to_stage(COLLISION_EVENT_HANDLING, block_handle_evader_collisions)
            .add_system_to_stage(COLLISION_EVENT_HANDLING, block_handle_obstacle_trigger_collisions)

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(block_vanish)
                    .with_system(block_spin)
                    .with_run_criteria(FixedTimestep::step(1.0))
                    .label(SystemLabels::UpdateWorld)
            )

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatch)
                    .with_system(block_despawn)
            )
        ;
    }
}


fn block_spawn(
    mut commands: Commands,
    my: Res<MyAssetPack>,
    empties: Query<(Entity, &Block), With<RequestTag>>,
    asset_server: Res<AssetServer>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltf_meshes: Res<Assets<GltfMesh>>,
    mut custom_materials: ResMut<Assets<BlockMaterial>>,
) {
    if let Some(gltf) = assets_gltf.get(&my.0) {
        let mesh =
            &assets_gltf_meshes.get(&gltf.named_meshes["SimpleBlock.001"]).unwrap()
                .primitives.get(0).unwrap().mesh;


        for (entity, block) in &empties {
            let mut block_commands = commands.entity(entity);


            // .with_scale(Vec3::new(BLOCK_WIDTH_H, BLOCK_HEIGHT / 4.0, BLOCK_DEPTH / 2.0))))
            block_commands
                .remove::<RequestTag>()
                .insert(Collider::round_cuboid(
                    BLOCK_WIDTH_H, BLOCK_HEIGHT / 4.0, BLOCK_DEPTH / 2.0,
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
                .insert(CollisionGroups::new(COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_DEATH | COLLIDER_GROUP_BALL | COLLIDER_GROUP_BLOCK | COLLIDER_GROUP_ARENA))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collidable {
                    kind: CollidableKind::Block
                })
            ;


            let group: TriggerGroup = match &block.trigger_group {
                None => 0,
                Some(g) => *g
            };

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

                BlockBehaviour::EvaderR(s) => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(s, 0.0, 0.0),
                    });
                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z));
                }

                BlockBehaviour::EvaderL(s) => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(-s, 0.0, 0.0),
                    });
                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z));
                }

                BlockBehaviour::EvaderU(s) => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(0.0, 0.0, -s),
                    });
                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_X));
                }

                BlockBehaviour::EvaderD(s) => {
                    block_commands.insert(BlockEvader {
                        velocity: Vec3::new(0.0, 0.0, s),
                    });

                    block_commands.insert(RigidBody::Dynamic);
                    block_commands.insert(LockedAxes::from(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_X));
                }

                _ => {}
            }


            match &block.trigger_type {
                None => {}
                Some(t) => match t {
                    TriggerType::ReceiverStartingInactive => {
                        block_commands.insert(BlockTriggerTarget { group });
                        block_commands.insert(BlockTriggerTargetInactive);
                    }

                    TriggerType::ReceiverStartingActive => {
                        block_commands.insert(BlockTriggerTarget { group });
                    }

                    _ => {
                        block_commands.insert(BlockTrigger {
                            group,
                            trigger_type: t.clone(),
                        });
                    }
                }
            };


            let mut color1 = Color::ORANGE;
            let mut color2 = Color::ORANGE;
            let mut top_bottom_split = false;

            match block.block_type {
                BlockType::Simple => {
                    block_commands.insert(Hittable {
                        hit_points: 1,
                        original_hit_points: 1,
                        ..default()
                    });
                }

                BlockType::Hardling => {
                    block_commands.insert(Hittable {
                        hit_points: 2,
                        original_hit_points: 2,
                        ..default()
                    });
                    color1 = Color::GRAY;
                }

                BlockType::Concrete => {
                    block_commands.insert(Hittable {
                        hit_points: 3,
                        original_hit_points: 3,
                        ..default()
                    });
                    color1 = Color::DARK_GRAY;
                }

                BlockType::SimpleTop => {
                    block_commands.insert(Hittable {
                        hit_points: 1,
                        original_hit_points: 1,
                        only_top: true,
                    });
                    top_bottom_split = true;
                    color1 = Color::ORANGE;
                    color2 = Color::WHITE;
                }

                BlockType::Obstacle => {
                    block_commands.insert({
                        Obstacle
                    });
                    color1 = Color::WHITE;
                }
            }

            let custom_material =
                custom_materials.add(BlockMaterial {
                    color1,
                    color2,
                    top_bottom_split,
                    color_texture: Some(asset_server.load("wreckage3.png")),
                    ..default()
                });

            block_commands
                .insert(MaterialMeshBundle {
                    mesh: mesh.clone(),
                    material: custom_material,
                    ..default()
                })
                .insert(TransformBundle::from(Transform::from_xyz(block.position.x, 0.0, block.position.y)))
            ;
        }
    }
}


fn block_update_custom_material(
    hittables: Query<(&Hittable, &Handle<BlockMaterial>)>,
    mut materials: ResMut<Assets<BlockMaterial>>,
) {
    for (hittable, block) in &hittables {
        if let Some(mut mat) = materials.get_mut(&block) {
            mat.damage = (hittable.original_hit_points - hittable.hit_points) as f32;
        }
    };
}


fn block_despawn(
    mut commands: Commands,
    blocks: Query<Entity, With<Block>>,
) {
    for block in &blocks {
        info!("Despawn block {:?}", block);
        commands.entity(block)
            .despawn_recursive();
    }
}

fn block_handle_obstacle_trigger_collisions(
    blocks: Query<(Entity, &BlockTrigger)>,
    collisions: Res<CollisionInfo>,
    mut triggerStates: ResMut<TriggerStates>,
) {
    for (block, trigger) in &blocks {
        let Some(collision) = collisions.collisions.get(&block) else { continue; };

        for collision in collision {
            match collision.other {
                CollidableKind::Ball => {
                    match trigger.trigger_type {
                        TriggerType::Start => triggerStates.start(trigger.group),
                        TriggerType::Stop => triggerStates.stop(trigger.group),
                        TriggerType::StartStop => triggerStates.flip(trigger.group),
                        TriggerType::ReceiverStartingInactive => {}
                        TriggerType::ReceiverStartingActive => {}
                    }
                }

                _ => {}
            }
        }
    }
}


fn block_handle_collisions(
    mut commands: Commands,
    mut blocks: Query<(Entity, &mut Hittable, &Block, &Transform), With<CollisionTag>>,
    mut events: EventWriter<MatchEvent>,
    collisions: Res<CollisionInfo>,
) {
    for (entity, mut hittable, block, trans) in &mut blocks {
        let Some(collision) = collisions.collisions.get(&entity) else { continue; };

        for collision in collision {
            match collision.other {
                CollidableKind::Ball => {
                    info!("{} {}", collision.other_pos.z, trans.translation.z);

                    if hittable.only_top == true && collision.other_pos.z < trans.translation.z
                        || hittable.only_top == false {
                        hittable.hit_points -= 1;
                    }


                    if hittable.hit_points == 0 {
                        commands.entity(entity)
                            .despawn_recursive();
                        events.send(MatchEvent::BlockHit(collision.pos.clone(), block.block_type.clone(), block.behaviour.clone()));
                    } else {
                        commands.entity(entity)
                            .insert(Shaking {
                                timer: Timer::from_seconds(Duration::from_millis(200).as_secs_f32(), TimerMode::Once),
                                original_position: trans.translation.clone(),
                                direction: if let Some(v) = collision.other_velocity { v.normalize() } else { Vec3::NEG_Z },
                            });
                    }
                }

                CollidableKind::DirectionalDeathTrigger(normal) => {
                    let dir = normal.dot(collision.pos - collision.other_pos);
                    if dir > 0.0 {
                        commands.entity(entity)
                            .despawn_recursive();
                        events.send(MatchEvent::BlockLost);
                    }
                }

                _ => {}
            }
        }
    }
}


fn block_handle_evader_collisions(
    mut commands: Commands,
    mut blocks: Query<(Entity, &mut BlockEvader), (With<Block>, With<CollisionTag>)>,
    mut events: EventWriter<MatchEvent>,
    collisions: Res<CollisionInfo>,
) {
    for (block, mut evader) in &mut blocks {
        if let Some(collision) = collisions.collisions.get(&block) {
            for collision in collision {
                match collision.other {
                    CollidableKind::Block | CollidableKind::Wall => {
                        evader.velocity *= -1.0;
                        info!("Flipping direction");
                    }

                    /* CollidableKind::DeathTrigger => {
                         commands.entity(block)
                             .despawn_recursive();
                         events.send(MatchEvent::BlockLost);

                         info!("Oh no! I died! {:?}", collision.other_pos);
                     }
 */
                    _ => {}
                }
            }
        }
    }
}


fn block_shake(
    mut commands: Commands,
    mut shaking: Query<(Entity, &mut Shaking, &mut Transform)>,
    time: Res<Time>,
) {
    for (block, mut shaking, mut trans) in &mut shaking {
        shaking.timer.tick(time.delta());
        if shaking.timer.percent() <= 0.5 {
            trans.translation += shaking.direction * shaking.timer.elapsed_secs();
        } else if !shaking.timer.just_finished() {
            trans.translation -= shaking.direction * shaking.timer.elapsed_secs() * 0.5;
        } else {
            commands.entity(block)
                .remove::<Shaking>();

            trans.translation = shaking.original_position;
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
        ball_force.force += bf;
    }
}


fn block_update_evader(
    time: Res<Time>,
    mut ball: Query<(&mut Transform, &mut BlockEvader), Without<BlockTriggerTargetInactive>>,
) {
    for (mut trans, mut evader) in &mut ball {
        trans.translation += evader.velocity * time.delta().as_secs_f32();
    }
}

fn block_update_trigger_targets(
    mut commands: Commands,
    mut targets: Query<(Entity, &BlockTriggerTarget)>,
    mut triggerStates: ResMut<TriggerStates>,
) {
    for (entity, target) in &targets {
        if triggerStates.is_started(target.group) &&
            !triggerStates.is_consumed(target.group) {
            info!("Starting receiver {:?}", target);
            commands.entity(entity)
                .remove::<BlockTriggerTargetInactive>();
            triggerStates.consume(target.group);
        } else if triggerStates.is_stopped(target.group) &&
            !triggerStates.is_consumed(target.group) {
            info!("Stopping receiver {:?}", target);
            commands.entity(entity)
                .insert(BlockTriggerTargetInactive);

            triggerStates.consume(target.group);
        }
    }
}