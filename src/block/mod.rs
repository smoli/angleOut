use bevy::prelude::{Commands, Component, Entity, EventReader, KeyCode, Query, Res, Transform, TransformBundle, With};
use bevy_rapier2d::geometry::CollisionGroups;
use bevy_rapier2d::prelude::{ActiveEvents, Collider, ContactForceEvent, Friction, Real, Restitution, RigidBody, Sensor};
use bevy_rapier2d::rapier::prelude::{CollisionEvent, CollisionEventFlags};
use crate::config::{BLOCK_HEIGHT_H, BLOCK_WIDTH_H, BLOCK_WIDTH, BLOCK_HEIGHT, MAX_RESTITUTION, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_BALL};


pub struct BlockHitEvent(Entity);

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
        .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
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

    println!("cx: {cx}  count: {count} gap: {gap} width: {BLOCK_WIDTH} -> x: {x}");
    for i in 0..count {
        spawn_block(commands, hit_points, x, y);
        x += BLOCK_WIDTH + gap;
    }
}

pub fn sys_handle_block_hit(mut commands: Commands, mut events: EventReader<ContactForceEvent>, mut query: Query<(Entity, &mut Block)>) {
    for ev in events.iter() {
        for (entity, mut block) in &mut query {
            if entity == ev.collider1 {
                println!("Block was hit(1)! {:?}", block)
            } else if entity == ev.collider2 {
                println!("Block was hit(2)! {:?}", block)
            } else {
                continue;
            }

            commands.entity(entity)
                .despawn();

        }
    }
}