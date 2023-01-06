use bevy::app::App;
use bevy::prelude::{Commands, Component, Entity, GamepadButtonType, KeyCode, Plugin, Query, Res, Transform, TransformBundle, Vec3, With, Without};
use bevy_rapier2d::dynamics::{ExternalImpulse, GravityScale, MassProperties, RigidBody, Velocity};
use bevy_rapier2d::geometry::{Collider, ColliderMassProperties, Friction, Restitution, Group};
use bevy::math::{Quat, Vec2};
use bevy::input::Input;
use bevy_rapier2d::math::Real;
use bevy_rapier2d::prelude::{CollisionGroups, LockedAxes};
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::{ActionState, InputMap};
use crate::actions::Action;
use crate::config::{BALL_SIZE, COLLIDER_GROUP_ARENA, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_NONE, COLLIDER_GROUP_PADDLE, MAX_BALL_SPEED, MAX_RESTITUTION, MIN_BALL_SPEED, PADDLE_THICKNESS, SCREEN_HEIGHT_H};
use crate::paddle_state::PaddleState;


#[derive(Component)]
pub struct Ball {}

#[derive(Component)]
pub struct ActiveBall;

#[derive(Component)]
pub struct InactiveBall;


pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(spawn_ball)

            .add_system(sys_update_ball_collision_group_active)
            .add_system(sys_update_inactive_ball)

            .add_system(sys_launch_inactive_ball)
            .add_system(sys_limit_ball_velocity)
        ;
    }
    
}

pub fn spawn_ball(mut commands: Commands) {
    commands
        .spawn(Ball {})
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.01))
        .insert(Collider::ball(BALL_SIZE))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Friction::coefficient(0.0))
        .insert(ColliderMassProperties::Density(20.0))
        .insert(ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Default::default(),
            mass: 1.0,
            principal_inertia: 0.0,

        }))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, SCREEN_HEIGHT_H, 0.0)))
        .insert(Velocity {
            linvel: Default::default(),
            angvel: 0.0,
        })
        .insert(ExternalImpulse {
            impulse: Vec2::new(0.0, 0.0),
            torque_impulse: 0.0,
        })

        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(GamepadButtonType::RightTrigger2, Action::LaunchBall)
                .build(),
        })
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(CollisionGroups::new(COLLIDER_GROUP_BALL, COLLIDER_GROUP_ARENA))
    ;
}

pub fn sys_update_ball_collision_group_active(mut query: Query<&mut CollisionGroups, With<ActiveBall>>) {
    for mut col in &mut query {
        col.filters = col.filters | COLLIDER_GROUP_PADDLE | COLLIDER_GROUP_BLOCK;
    }
}

pub fn sys_update_inactive_ball(paddleState: Res<PaddleState>, mut query: Query<(&mut Transform, &mut CollisionGroups), (Without<ActiveBall>, With<Ball>)>) {
    for (mut trans, mut col) in &mut query {
        col.filters = col.filters & !COLLIDER_GROUP_PADDLE;
        col.filters = col.filters & !COLLIDER_GROUP_BLOCK;
        trans.translation = paddleState.paddle_position.clone() + Vec3::new(0.0, PADDLE_THICKNESS + 1.5 * BALL_SIZE, 0.0);
    }
}


pub fn determine_launch_impulse(angle: Real, value: Real) -> Vec2 {
    let mut imp = Vec3::new(0.0, value, 0.0);

    println!("Launch for {angle} {:?}", imp);

    let r = Quat::from_rotation_z(-angle) * imp;

    Vec2::new(r.x, r.y)
}

pub fn sys_launch_inactive_ball(paddleState: Res<PaddleState>, mut commands: Commands, mut query: Query<(Entity, &ActionState<Action>, &mut ExternalImpulse), (Without<ActiveBall>, With<Ball>)>) {
    for (ball, action, mut impluse) in &mut query {
        if !action.pressed(Action::LaunchBall) { continue; }


        impluse.impulse = determine_launch_impulse(paddleState.paddle_rotation, 1000.0);

        commands.entity(ball)
            .insert(ActiveBall {});
    }
}


pub fn sys_limit_ball_velocity(mut query: Query<&mut Velocity, With<ActiveBall>>) {
    for (mut velo) in &mut query {
        let v = velo.linvel.length();

        if v > MAX_BALL_SPEED {
            velo.linvel = velo.linvel / v * MAX_BALL_SPEED;
        } else if v < MIN_BALL_SPEED {
            velo.linvel = velo.linvel / v * MIN_BALL_SPEED;
        }
    }
}
