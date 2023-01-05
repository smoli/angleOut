use bevy::prelude::{Commands, Component, KeyCode, Query, Res, Transform, TransformBundle, With};
use bevy_rapier2d::dynamics::{ExternalImpulse, GravityScale, MassProperties, RigidBody, Velocity};
use bevy_rapier2d::geometry::{Collider, ColliderMassProperties, Friction, Restitution};
use bevy::math::Vec2;
use bevy::input::Input;
use crate::config::{BALL_SIZE, MAX_BALL_SPEED, MAX_RESTITUTION};


#[derive(Component)]
pub struct Ball {
    launching: bool,
}


pub fn spawn_ball(mut commands: Commands) {
    /* Create the bouncing ball. */
    commands
        .spawn(Ball { launching: false })
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(Collider::ball(BALL_SIZE))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))
        .insert(ColliderMassProperties::Density(20.0))
        .insert(ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Default::default(),
            mass: 2.0,
            principal_inertia: 0.0,

        }))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
        .insert(Velocity {
            linvel: Default::default(),
            angvel: 0.0,
        })
        .insert(ExternalImpulse {
            impulse: Vec2::new(0.0, 0.0),
            torque_impulse: 0.0,
        });
}


pub fn sys_apply_force_to_ball_on_space(
    input: Res<Input<KeyCode>>,
    mut impulse: Query<&mut ExternalImpulse>) {
    if !input.just_pressed(KeyCode::Space) { return; }


    let mut ef = impulse.single_mut();

    ef.impulse = Vec2::new(200.0, 200.0);
}


pub fn sys_limit_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    for (mut velo) in &mut query {
        let v = velo.linvel.length();

        if v > MAX_BALL_SPEED {
            velo.linvel = velo.linvel / v * MAX_BALL_SPEED;
        }
    }
}
