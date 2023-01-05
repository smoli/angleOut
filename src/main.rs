#[allow(unused_imports)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::prelude::KeyCode::{Ax, V};
use bevy::window::WindowResizeConstraints;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::na::point;
use bevy_rapier2d::parry::math::AngularInertia;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::dynamics::RigidBodyBuilder;
use bevy_rapier2d::rapier::prelude::RigidBodyType;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::prelude::DualAxis;

const PIXELS_PER_METER: f32 = 100.0;

const SCREEN_WIDTH: Real = 1000.0;
const SCREEN_HEIGHT: Real = 500.0;
const SCREEN_WIDTH_H: Real = SCREEN_WIDTH / 2.0;
const SCREEN_HEIGHT_H: Real = 500.0 / 2.0;
const BALL_SIZE: Real = 10.0;
const MAX_BALL_SPEED: Real = 500.0;
const MAX_RESTITUTION: Real = 1.0;

const PADDLE_WIDTH: Real = 150.0;
const PADDLE_WIDTH_H: Real = PADDLE_WIDTH / 2.0;
const PADDLE_THICKNESS: Real = 10.0;
const PADDLE_LIFT: Real = PADDLE_THICKNESS * 3.0;


// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    ArticulateLeft,
    ArticulateRight,
}

#[derive(Component)]
struct Ball {
    launching: bool,
}

#[derive(Component)]
struct Paddle {
    target_position: Vec2,
    target_rotation: Real,
    current_rotation: Real
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))

        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -100.0),
            ..default()
        })

        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
                position: WindowPosition::Centered,
                monitor: MonitorSelection::Current,
                title: "Angle Out".to_string(),
                ..default()
            },
            ..default()
        }))
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())

        .add_plugin(InspectableRapierPlugin)
        .add_plugin(WorldInspectorPlugin::default())

        .add_plugin(InputManagerPlugin::<Action>::default())

        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(spawn_camera)

        .add_startup_system(spawn_arena)

        .add_startup_system(spawn_ball)

        .add_startup_system(spawn_paddle)

        .add_system(sys_apply_force_to_ball_on_space)

        .add_system(sys_articulate_paddle)
        .add_system(sys_update_paddle_position)

        .add_system(sys_limit_ball_velocity)

        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_paddle(mut commands: Commands) {
    commands
        .spawn(RigidBody::KinematicPositionBased)
        .insert(Paddle {
            target_position: Default::default(),
            target_rotation: 0.0,
            current_rotation: 0.0,
        })

        .insert(Collider::cuboid(PADDLE_WIDTH_H, PADDLE_THICKNESS))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -SCREEN_HEIGHT_H + PADDLE_LIFT, 0.0)))
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


fn spawn_arena(mut commands: Commands) {
    commands
        .spawn(Collider::cuboid(SCREEN_WIDTH_H, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -SCREEN_HEIGHT_H, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(SCREEN_WIDTH_H, 10.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, SCREEN_HEIGHT_H, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, SCREEN_HEIGHT_H))
        .insert(TransformBundle::from(Transform::from_xyz(-SCREEN_WIDTH_H, 0.0, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));

    commands
        .spawn(Collider::cuboid(10.0, SCREEN_HEIGHT_H))
        .insert(TransformBundle::from(Transform::from_xyz(SCREEN_WIDTH_H, 0.0, 0.0)))
        .insert(Friction::coefficient(0.0))
        .insert(Restitution::coefficient(MAX_RESTITUTION));
}

fn spawn_ball(mut commands: Commands) {
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


fn sys_apply_force_to_ball_on_space(
    input: Res<Input<KeyCode>>,
    mut impulse: Query<&mut ExternalImpulse>) {
    if !input.just_pressed(KeyCode::Space) { return; }


    let mut ef = impulse.single_mut();

    ef.impulse = Vec2::new(200.0, 200.0);
}


fn sys_limit_ball_velocity(mut query: Query<&mut Velocity, With<Ball>>) {
    for (mut velo) in &mut query {
        let v = velo.linvel.length();

        if v > MAX_BALL_SPEED {
            velo.linvel = velo.linvel / v * MAX_BALL_SPEED;
        }
    }
}

const PADDLE_RESTING_Y: Real = -SCREEN_HEIGHT_H + PADDLE_LIFT;
const PADDLE_RESTING_X: Real = 0.0;
const PADDLE_RESTING_ROTATION: Real = 0.0;

fn sys_articulate_paddle(mut query: Query<(&mut Transform, &ActionState<Action>, &mut Paddle)>) {
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
            comp.x * (SCREEN_WIDTH_H - PADDLE_WIDTH_H - PADDLE_THICKNESS)
        };

        let ty = comp.y * PADDLE_LIFT - SCREEN_HEIGHT_H + PADDLE_LIFT;

        paddle.target_position = Vec2::new(tx, ty);
    }
}

const PADDLE_ROTATION_ACCEL:Real = 10.0;
const PADDLE_POSITION_ACCEL:Real = 10.0;

fn sys_update_paddle_position(time: Res<Time>, mut query: Query<(&mut Transform, &mut Paddle)>) {
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


fn sys_gamepad_info(
    gamepads: Res<Gamepads>,
    button_inputs: Res<Input<GamepadButton>>,
    button_axes: Res<Axis<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
) {
    for gamepad in gamepads.iter() {
        if button_inputs.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South)) {
            info!("{:?} just pressed South", gamepad);
        } else if button_inputs.just_released(GamepadButton::new(gamepad, GamepadButtonType::South))
        {
            info!("{:?} just released South", gamepad);
        }

        let right_trigger = button_axes
            .get(GamepadButton::new(
                gamepad,
                GamepadButtonType::RightTrigger2,
            ))
            .unwrap();
        if right_trigger.abs() > 0.01 {
            info!("{:?} RightTrigger2 value is {}", gamepad, right_trigger);
        }

        let left_stick_x = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        if left_stick_x.abs() > 0.01 {
            info!("{:?} LeftStickX value is {}", gamepad, left_stick_x);
        }
    }
}