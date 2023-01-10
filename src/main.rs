mod config;
mod r#match;
mod state;
mod events;
mod labels;
mod ui;
mod actions;
mod ship;
mod arena;
mod ball;
mod physics;
mod block;

use std::f32::consts::PI;
use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::math::Quat;
use bevy::pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle};
use bevy::prelude::{AssetServer, Camera, Camera2dBundle, Camera3dBundle, ClearColor, Color, Commands, GamepadButtonType, OrthographicProjection, PluginGroup, Query, Res, SceneBundle, Transform, Vec3, WindowDescriptor, With};
use bevy::utils::default;
use bevy::window::{close_on_esc, MonitorSelection, WindowPlugin, WindowPosition};
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::RapierDebugRenderPlugin;
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::{ActionState, InputManagerPlugin, InputMap};
use crate::actions::{CameraActions, GameFlowActions, MatchActions};
use crate::arena::ArenaPlugin;
use crate::ball::BallPlugin;
use crate::block::BlockPlugin;
use crate::config::{PIXELS_PER_METER, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::events::{EventsPlugin, GameFlowEvent, MatchEvent};
use crate::physics::PhysicsPlugin;
use crate::ship::ShipPlugin;
use crate::state::GameState;
use crate::ui::UI;

fn main() {
    let mut app = App::new();

    setup_screen(&mut app);
    setup_ui(&mut app);
    app.add_plugin(EventsPlugin);

    app.add_state(GameState::InGame);

    app.add_startup_system(setup_3d_environment);
    app.add_system(camera_update_position);

    app.add_plugin(PhysicsPlugin);

    app.add_plugin(ShipPlugin);
    app.add_plugin(ArenaPlugin);
    app.add_plugin(BallPlugin);
    app.add_plugin(BlockPlugin);

    app.add_plugin(InputManagerPlugin::<GameFlowActions>::default());
    app.add_plugin(InputManagerPlugin::<MatchActions>::default());
    app.add_plugin(InputManagerPlugin::<CameraActions>::default());

    app.add_system(close_on_esc);
    app.run();
}

fn setup_screen(app: &mut App) {
    // app.insert_resource(ClearColor(Color::BLACK))
    app.insert_resource(ClearColor(Color::ALICE_BLUE))
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
    }));
}

fn camera_update_position(mut query: Query<(&mut Transform, &mut ActionState<CameraActions>), With<Camera>>) {
    for (mut trans, mut action) in &mut query {

        let mut rotation:Option<Quat> = None;

        if action.pressed(CameraActions::Down) {
            rotation = Some(Quat::from_rotation_x(PI / 20.0));
            // action.consume(CameraActions::Down);
        }
        if action.pressed(CameraActions::Up) {
            rotation = Some(Quat::from_rotation_x(-PI / 20.0));
            // action.consume(CameraActions::Up);
        }
        if action.pressed(CameraActions::Left) {
            rotation = Some(Quat::from_rotation_y(-PI / 20.0));
            // action.consume(CameraActions::Left);
        }

        if action.pressed(CameraActions::Right) {
            rotation = Some(Quat::from_rotation_y(PI / 20.0));
            // action.consume(CameraActions::Right);
        }


        if let Some(r) = rotation {
            let v = trans.translation.clone();
            let v2 = r.mul_vec3(v);
            let nt = Transform::from_xyz(v2.x, v2.y, v2.z).looking_at(Vec3::ZERO, Vec3::Y);

            trans.translation = nt.translation;
            trans.rotation = nt.rotation;
        }

        if action.pressed(CameraActions::Reset) {
            let nt = Transform::from_xyz(0.0, 20.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y);

            trans.translation = nt.translation;
            trans.rotation = nt.rotation;
        }

    }
}

fn setup_3d_environment(
    mut commands: Commands,
) {
    // commands.spawn(Camera2dBundle::default());
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),
        // transform: Transform::from_xyz(0.0, 0.0, -100.00001).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
        .insert(InputManagerBundle::<CameraActions> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(GamepadButtonType::North, CameraActions::Reset)
                .insert(GamepadButtonType::DPadDown, CameraActions::Down)
                .insert(GamepadButtonType::DPadUp, CameraActions::Up)
                .insert(GamepadButtonType::DPadLeft, CameraActions::Left)
                .insert(GamepadButtonType::DPadRight, CameraActions::Right)

                .build(),
        })
    ;

    // Directional Light
    const HALF_SIZE: f32 = 30.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.7, 0.7, 1.0),
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadow_depth_bias: 0.0,
            shadows_enabled: true,
            illuminance: 75_000.0,
            ..default()

        },
        transform: Transform::from_xyz(200.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),

        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.7, 0.7, 1.0),
            shadow_depth_bias: 0.0,
            shadows_enabled: false,
            illuminance: 5_000.0,
            ..default()
        },
        transform: Transform::from_xyz(-200.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),

        ..default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.4,
    });



}

fn setup_ui(app: &mut App) {
    app.add_plugin(UI);
}

