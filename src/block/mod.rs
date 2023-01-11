use bevy::utils::default;
use bevy::app::App;
use bevy::log::{info};
use bevy::math::Vec2;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::{AssetServer, Commands, Component, DespawnRecursiveExt, Entity, IntoSystemDescriptor, Plugin, Query, Res, SceneBundle, SystemSet, Transform, TransformBundle, With};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionGroups, Friction, Restitution, RigidBody};
use crate::config::{BLOCK_DEPTH, BLOCK_HEIGHT, BLOCK_ROUNDNESS, BLOCK_WIDTH, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, MAX_RESTITUTION};
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::physics::{Collidable, CollidableKind, CollisionTag};
use crate::state::GameState;


#[derive(Component)]
pub struct Block {
    pub position: Vec2,
    pub asset_name: String
}


impl Default for Block {
    fn default() -> Self {
        Block {
            position: Default::default(),
            asset_name: "ship3_003.gltf#Scene3".to_string(),
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
        ;
    }
}


fn block_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    empties: Query<(Entity, &Block), With<RequestTag>>
) {
    for (entity, block) in &empties {
        commands.entity(entity)
            .remove::<RequestTag>()
            .insert(RigidBody::Fixed)

            .insert(SceneBundle {
                scene: asset_server.load(block.asset_name.as_str()),
                ..default()
            })
            .insert(TransformBundle::from(Transform::from_xyz(block.position.x, 0.0, block.position.y)))
            .insert(Collider::round_cuboid(
                BLOCK_WIDTH / 2.0 - BLOCK_ROUNDNESS,
                BLOCK_HEIGHT * 2.0 - BLOCK_ROUNDNESS,
                BLOCK_DEPTH / 2.0 - BLOCK_ROUNDNESS,
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
    }



}

fn block_handle_collisions(
    mut commands: Commands,
    mut blocks: Query<(Entity, &CollisionTag), With<Block>>) {
    for (block, collision) in &mut blocks {
        match collision.other {
            CollidableKind::Ball => {
                commands.entity(block)
                    .despawn_recursive();
            }

            _ => {}
        }
    }
}
