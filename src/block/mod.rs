use bevy::prelude::{Commands, Component, KeyCode, Query, Res, Transform, TransformBundle, With};
use bevy_rapier2d::prelude::{Collider, Real, Restitution, RigidBody};
use crate::config::{BLOCK_HEIGHT_H, BLOCK_WIDTH_H, BLOCK_WIDTH, BLOCK_HEIGHT, MAX_RESTITUTION};


#[derive(Component)]
pub struct Block {
    hit_points: usize
}

pub fn spawn_block(commands: &mut Commands, hit_points: usize, x: Real, y: Real) {

    commands
        .spawn(Block {
            hit_points,
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid( BLOCK_WIDTH_H, BLOCK_HEIGHT_H))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(TransformBundle::from(Transform::from_xyz(x, y, 0.0)))
    ;

}

pub fn spawn_block_row(commands:&mut  Commands, hit_points: usize, cx: Real, y: Real, gap: Real, count: usize) {

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
