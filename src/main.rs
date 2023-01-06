mod paddle;
mod ball;
mod arena;
mod config;
mod actions;
mod block;
mod paddle_state;

#[allow(unused_imports)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::query::{QueryEntityError, ReadOnlyWorldQuery, WorldQuery};
use bevy::ecs::system::WorldState;
use bevy::prelude::*;
use bevy::prelude::KeyCode::{Ax, B, V};
use bevy::utils::tracing::event;
use bevy::window::WindowResizeConstraints;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy_prototype_lyon::prelude::{DrawMode, GeometryBuilder, Path, PathBuilder, StrokeMode};
use bevy_rapier2d::na::point;
use bevy_rapier2d::parry::math::AngularInertia;
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::rapier::dynamics::RigidBodyBuilder;
use bevy_rapier2d::rapier::prelude::RigidBodyType;
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::prelude::DualAxis;
use actions::Action;
use config::{BLOCK_GAP, BLOCK_HEIGHT, BLOCK_WIDTH, PIXELS_PER_METER, SCREEN_HEIGHT, SCREEN_HEIGHT_H, SCREEN_WIDTH, SCREEN_WIDTH_H};
use block::{spawn_block, spawn_block_row};
use ball::{sys_update_ball_collision_group_active, sys_update_inactive_ball};
use paddle_state::PaddleState;
use crate::ball::{BallPlugin, sys_launch_inactive_ball};
use crate::block::{Block, BlockHitState, sys_handle_block_hit};
use crate::paddle::{Paddle, PaddlePlugin, sys_bounce_ball_from_paddle};

fn main() {
    App::new()
        .insert_resource(PaddleState {
            paddle_rotation: 0.0,
            paddle_position: Default::default(),
        })

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


        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER))

        .add_plugin(InputManagerPlugin::<Action>::default())

        .add_plugin(BallPlugin)
        .add_plugin(PaddlePlugin)

        .add_startup_system(spawn_camera)

        .add_startup_system(arena::spawn_arena)

        .add_startup_system(system_spawn_blocks)


        .add_system(handle_collision_events)
        .add_system(sys_handle_block_hit)

        /* Debug Stuff */
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(WorldInspectorPlugin::default())
        // .add_plugin(InspectableRapierPlugin)
        // .add_plugin(ShapePlugin)
        // .add_startup_system(spawn_paddle_normal)
        // .add_system(system_adjust_paddle_normal)

        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn system_spawn_blocks(mut commands: Commands) {
    for i in 0..5 {
        spawn_block_row(&mut commands, 1, 0.0, i as Real * (BLOCK_HEIGHT + BLOCK_GAP) + BLOCK_HEIGHT, BLOCK_GAP, 7);
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


fn tag_collision_both<F: ReadOnlyWorldQuery>(
    commands: &mut Commands,
    event: &CollisionEvent,
    query: &Query<Entity, F>,
    tag: impl Component) -> bool
{
    match event {
        CollisionEvent::Started(a, b, _) => {
            match query.get(*a) {
                Ok(e) => {
                    commands.entity(e)
                        .insert(tag);
                    true
                }
                Err(_) => match query.get(*b) {
                    Ok(e) => {
                        commands.entity(e)
                            .insert(tag);
                        true
                    }
                    Err(_) => false
                }
            }
        }
        _ => false
    }
}

fn handle_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    block: Query<Entity, With<Block>>,
    paddle: Query<Entity, With<Paddle>>,
) {
    for collision_event in collision_events.iter() {
        // println!("Received collision event: {:?}", collision_event);
        // println!("{:?}", collision_event);

        if tag_collision_both::<With<Block>>(&mut commands, collision_event, &block, BlockHitState {}) {
            println!("Tagged Block");
        } else if tag_collision_both::<With<Paddle>>(&mut commands, collision_event, &paddle, BlockHitState {}) {
            println!("Tagged Paddle");
        }
    }
}

fn spawn_paddle_normal(mut commands: Commands) {
    let mut path = PathBuilder::new();
    path.move_to(Vec2::ZERO);
    path.line_to(Vec2::new(0.0, 100.0));
    let line = path.build();
    commands.spawn(GeometryBuilder::build_as(
        &line,
        DrawMode::Stroke(StrokeMode::new(Color::LIME_GREEN, 10.0)),
        Transform::default(),
    ));
}

fn system_adjust_paddle_normal(paddleState: Res<PaddleState>, mut paths: Query<&mut Transform, With<Path>>) {
    for mut p in &mut paths {
        p.rotation = Quat::from_rotation_z(-paddleState.paddle_rotation);
    }
}