use bevy::prelude::{AssetServer, Audio, Commands, Component, Entity, EventReader, KeyCode, Query, Res, ResMut, Transform, TransformBundle, With};
use bevy_rapier2d::geometry::CollisionGroups;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, ContactForceEvent, Friction, Real, Restitution, RigidBody, Sensor};
use bevy_rapier2d::rapier::prelude::{CollisionEvent, CollisionEventFlags};
use crate::config::{BLOCK_HEIGHT_H, BLOCK_WIDTH_H, BLOCK_WIDTH, BLOCK_HEIGHT, MAX_RESTITUTION, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_BALL};
use crate::paddle::Paddle;
use crate::states::GameState;


#[derive(Component, Debug)]
pub struct BlockHitState;

#[derive(Component, Debug)]
pub struct Block {
    hit_points: usize,
}

pub fn spawn_block(commands: &mut Commands, hit_points: usize, x: Real, y: Real) {
    commands
        .spawn(Block {
            hit_points,
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(BLOCK_WIDTH_H, BLOCK_HEIGHT_H))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(TransformBundle::from(Transform::from_xyz(x, y, 0.0)))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))
        .insert(CollisionGroups::new(COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_BALL))
        .insert(ActiveEvents::COLLISION_EVENTS)
    ;
}

pub fn spawn_block_row(commands: &mut Commands, hit_points: usize, cx: Real, y: Real, gap: Real, count: usize) {
    let ct = (count / 2) as Real;

    let mut x: Real = cx - ct * (BLOCK_WIDTH + gap) - gap + BLOCK_WIDTH_H;

    if count % 2 == 1 {
        x -= BLOCK_WIDTH_H - gap;
    } else {
        x += gap * 1.5;
    }

    for i in 0..count {
        spawn_block(commands, hit_points, x, y);
        x += BLOCK_WIDTH + gap;
    }
}

pub fn sys_handle_block_hit(
    mut gameState: ResMut<GameState>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Block), With<BlockHitState>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>
)
{
    for (entity, mut block) in &mut query {

        block.hit_points -= 1;
        if block.hit_points > 0 {
            commands.entity(entity).remove::<BlockHitState>();
        } else {
            commands.entity(entity).despawn();
            gameState.subBlocks(1);
            let boom = asset_server.load("explosionCrunch_000.ogg");
            audio.play(boom);
        }
    }

}