use bevy::prelude::{Commands, Component, Query, Res, Time, Transform, TransformBundle};
use bevy_rapier2d::dynamics::{MassProperties, RigidBody};
use bevy_rapier2d::geometry::{Collider, ColliderMassProperties, Friction, Restitution};
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::axislike::{DualAxis, DualAxisData};
use bevy::math::{Quat, Vec2};
use bevy_rapier2d::math::Real;
use crate::actions::Action;
use crate::config::{ARENA_HEIGHT_H, ARENA_WIDTH_H, MAX_RESTITUTION, PADDLE_LIFT, PADDLE_POSITION_ACCEL, PADDLE_RESTING_ROTATION, PADDLE_RESTING_X, PADDLE_RESTING_Y, PADDLE_ROTATION_ACCEL, PADDLE_THICKNESS, PADDLE_WIDTH_H, SCREEN_HEIGHT_H, SCREEN_WIDTH_H};


#[derive(Component)]
pub struct Paddle {
    target_position: Vec2,
    target_rotation: Real,
    current_rotation: Real,
}


pub fn spawn_paddle(mut commands: Commands) {
    commands
        .spawn(RigidBody::KinematicPositionBased)
        .insert(Paddle {
            target_position: Default::default(),
            target_rotation: 0.0,
            current_rotation: 0.0,
        })

        .insert(Collider::cuboid(PADDLE_WIDTH_H, PADDLE_THICKNESS))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -ARENA_HEIGHT_H + PADDLE_LIFT, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(ColliderMassProperties::Density(20.0))
        .insert(ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Default::default(),
            mass: 2.0,
            principal_inertia: 0.0,

        }))
        /*  .insert(Velocity {
              linvel: Vec2::new(0.0, 0.0),
              angvel: 0.0,
          })*/
        // .insert(Dominance::group(100))
        .insert(Restitution::coefficient(MAX_RESTITUTION))

        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::ArticulateLeft)
                .insert(DualAxis::right_stick(), Action::ArticulateRight)
                .build(),
        });
}


pub fn sys_articulate_paddle(mut query: Query<(&mut Transform, &ActionState<Action>, &mut Paddle)>) {
    for (mut trans, action_state, mut paddle) in &mut query {
        if !action_state.pressed(Action::ArticulateLeft) && !action_state.pressed(Action::ArticulateRight) {
            paddle.target_position = Vec2::new(PADDLE_RESTING_X, PADDLE_RESTING_Y);
            paddle.target_rotation = PADDLE_RESTING_ROTATION;
            return;
        }

        let axis_pair_l: DualAxisData = action_state.clamped_axis_pair(Action::ArticulateLeft).unwrap();
        let axis_pair_r: DualAxisData = action_state.clamped_axis_pair(Action::ArticulateRight).unwrap();

        // Rotation
        let mut d = Vec2::new(-1.0, axis_pair_l.y()) - Vec2::new(1.0, axis_pair_r.y());

        let mut a = d.perp().angle_between(Vec2::new(0.0, -1.0));
        if a.abs() < 0.1 { a = PADDLE_RESTING_ROTATION }

        paddle.target_rotation = a;


        // Translation
        let comp = (axis_pair_l.xy() + axis_pair_r.xy()) * 0.5;

        let tx = if comp.length() < 0.2 {
            PADDLE_RESTING_X
        } else {
            comp.x * (ARENA_WIDTH_H - PADDLE_WIDTH_H - PADDLE_THICKNESS)
        };

        let ty = comp.y * PADDLE_LIFT - ARENA_HEIGHT_H + PADDLE_LIFT;

        paddle.target_position = Vec2::new(tx, ty);
    }
}


pub fn sys_update_paddle_position(time: Res<Time>, mut query: Query<(&mut Transform, &mut Paddle)>) {
    for (mut trans, mut paddle) in &mut query {
        let dp = paddle.target_position.extend(trans.translation.z) - trans.translation;

        let mut tp = paddle.target_position.extend(trans.translation.z);
        if dp.length() > 0.01 {
            tp = trans.translation + dp * time.delta_seconds() * PADDLE_POSITION_ACCEL;
        }

        trans.translation = tp;

        let dr = paddle.target_rotation - paddle.current_rotation;

        let mut a = paddle.target_rotation;
        if dr.abs() > 0.001 {
            a = paddle.current_rotation + dr * time.delta_seconds() * PADDLE_ROTATION_ACCEL;
        }
        paddle.current_rotation = a;
        trans.rotation = Quat::from_rotation_z(-a);
    }
}
