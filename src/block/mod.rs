use bevy::utils::default;
use bevy::app::App;
use bevy::log::{info};
use bevy::prelude::{AssetServer, Commands, Component, DespawnRecursiveExt, Entity, IntoSystemDescriptor, Plugin, Query, Res, SceneBundle, SystemSet, Transform, TransformBundle, With};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionGroups, Friction, Restitution, RigidBody};
use crate::config::{COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, MAX_RESTITUTION};
use crate::labels::SystemLabels;
use crate::physics::{Collidable, CollidableKind, CollisionTag};
use crate::state::GameState;


#[derive(Component)]
pub struct Block;


pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(blocks_spawn)
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(handle_block_collisions.label(SystemLabels::UpdateWorld))
            )
        ;
    }
}

const BLOCK_WIDTH: f32 = 0.75;
const BLOCK_HEIGHT: f32 = 0.187;
const BLOCK_DEPTH: f32 = 0.375;
const BLOCK_ROUNDNESS: f32 = 0.02;


fn int_spawn_one_block(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    x: f32,
    y: f32,
) {
    commands
        .spawn(RigidBody::Fixed)
        .insert(SceneBundle {
            scene: asset_server.load("ship3_003.gltf#Scene3"),
            ..default()
        })
        .insert(Collider::round_cuboid(
            BLOCK_WIDTH / 2.0 - BLOCK_ROUNDNESS,
            BLOCK_HEIGHT / 2.0 - BLOCK_ROUNDNESS,
            BLOCK_DEPTH / 2.0 - BLOCK_ROUNDNESS,
            BLOCK_ROUNDNESS,
        ))
        .insert(TransformBundle::from(Transform::from_xyz(x, 0.0, y)))

        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))
        .insert(CollisionGroups::new(COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_BALL))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Collidable {
            kind: CollidableKind::Block
        })
        .insert(Block)

    ;
}


fn blocks_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let row_count = 10;
    let count = 17;
    let gap: f32 = 0.3;

    let count_h = count as f32 / 2.0;

    let step = BLOCK_WIDTH + gap;

    let mut y = -3.0;

    for _ in 0..row_count {
        let mut x = -step * count_h + gap;
        for _ in 0..count {
            info!("{x}");
            int_spawn_one_block(&mut commands, &asset_server, x, y);
            x += step;
        }

        y -= BLOCK_DEPTH * 2.0 - gap;
    }
}

fn handle_block_collisions(
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
