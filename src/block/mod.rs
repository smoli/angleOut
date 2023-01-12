use std::f32::consts::TAU;
use bevy::utils::default;
use bevy::app::App;
use bevy::log::{info};
use bevy::math::Vec2;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::{AssetServer, Commands, Component, DespawnRecursiveExt, Entity, IntoSystemDescriptor, Plugin, Query, Res, SceneBundle, SystemSet, Time, Transform, TransformBundle, With};
use bevy::time::FixedTimestep;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionGroups, Friction, Restitution, RigidBody};
use crate::config::{BLOCK_DEPTH, BLOCK_HEIGHT, BLOCK_ROUNDNESS, BLOCK_WIDTH, BLOCK_WIDTH_H, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, MAX_RESTITUTION};
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::physics::{Collidable, CollidableKind, CollisionTag};
use crate::state::GameState;


#[derive(Debug)]
pub enum BlockType {
    Simple,
    Hardling,
    Concrete,
}

#[derive(Debug)]
pub enum BlockBehaviour {
    SittingDuck,
    Spinner,
    EvadeUp
}


#[derive(Component)]
struct BlockSpinner;

#[derive(Component)]
struct BlockEvadeUp;

#[derive(Component, Debug)]
pub struct Block {
    pub position: Vec2,
    pub asset_name: String,
    pub block_type: BlockType,
    pub behaviour: BlockBehaviour,
}

#[derive(Component, Debug)]
pub struct Hittable {
    pub hit_points: u8
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

            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(block_spawn.label(SystemLabels::UpdateWorld))
                    .with_system(block_handle_collisions.label(SystemLabels::UpdateWorld))
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(block_spin)
                    .with_system(block_evade_up)
                        .with_run_criteria(FixedTimestep::step(1.0))
                        .label(SystemLabels::UpdateWorld)
            )
        ;
    }
}


fn block_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    empties: Query<(Entity, &Block), With<RequestTag>>,
) {
    for (entity, block) in &empties {
        let mut block_commands = commands.entity(entity);

        block_commands
            .remove::<RequestTag>()
            .insert(RigidBody::Fixed)

            .insert(SceneBundle {
                scene: asset_server.load(block.asset_name.as_str()),
                ..default()
            })
            .insert(TransformBundle::from(Transform::from_xyz(block.position.x, 0.0, block.position.y)))
            .insert(Collider::round_cuboid(
                BLOCK_WIDTH / 2.0 - 2.0 * BLOCK_ROUNDNESS,
                BLOCK_HEIGHT / 2.0 - 2.0 * BLOCK_ROUNDNESS,
                BLOCK_DEPTH / 2.0 - 2.0 * BLOCK_ROUNDNESS,
                BLOCK_ROUNDNESS,
            ))

            .insert(Restitution::coefficient(MAX_RESTITUTION))
            .insert(Friction::coefficient(0.0))
            .insert(CollisionGroups::new(COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_BALL))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Collidable {
                kind: CollidableKind::Block
            })
        ;


        match block.behaviour {
            BlockBehaviour::Spinner => {
                block_commands.insert(BlockSpinner);
            }

            BlockBehaviour::EvadeUp => {
                block_commands.insert(BlockEvadeUp);
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

fn block_handle_collisions(
    mut commands: Commands,
    mut blocks: Query<(Entity, &CollisionTag, &mut Hittable), With<Block>>) {
    for (entity, collision, mut hittable) in &mut blocks {
        match collision.other {
            CollidableKind::Ball => {
                hittable.hit_points -= 1;

                if hittable.hit_points == 0 {
                    commands.entity(entity)
                        .despawn_recursive();
                } else {
                    info!("still alive")
                }
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


fn block_evade_up(
    mut ball: Query<&mut Transform, With<BlockEvadeUp>>) {
    for mut trans in &mut ball {
        trans.translation.x -= BLOCK_WIDTH_H
    }
}