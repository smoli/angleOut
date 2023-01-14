pub mod state;

use std::f32::consts::PI;
use bevy::app::App;
use bevy::prelude::{AmbientLight, Camera, Camera3dBundle, Color, Commands, Component, default, DirectionalLight, DirectionalLightBundle, Entity, EventReader, GamepadButtonType, IntoSystemDescriptor, OrthographicProjection, Plugin, Quat, Query, ResMut, SystemSet, Transform, Vec3, With};
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::{ActionState, InputMap};
use crate::actions::CameraActions;
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::r#match::state::MatchState;
use crate::state::GameState;
use crate::ui::{Environment3d, tear_down_3d_environment};

#[derive(Component)]
pub struct Match;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(state::MatchState::default())

            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(match_spawn.before(SystemLabels::UpdateWorld))
                    .with_system(setup_3d_environment.before(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(camera_update_position)
            )

            .add_system_set(
                SystemSet::on_exit(GameState::InMatch)
                    .with_system(match_despawn)
                    .with_system(tear_down_3d_environment)
            );
    }
}


fn match_spawn(
    mut match_state: ResMut<MatchState>,
    mut commands: Commands
) {
    match_state.reset();
    commands.spawn(Match);
}

fn match_despawn(mut commands: Commands, matches: Query<Entity, With<Match>>) {
    for the_match in &matches {
        commands.entity(the_match).despawn();
    }
}


fn setup_3d_environment(
    mut commands: Commands,
) {
    // commands.spawn(Camera2dBundle::default());
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),
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
        }).insert(Environment3d);
    ;

    // Directional Light
    const HALF_SIZE: f32 = 300.0;
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
            illuminance: 75_000.0 / 2.0,
            ..default()

        },
        transform: Transform::from_xyz(200.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),

        ..default()
    }).insert(Environment3d);

    // Directional Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.7, 0.7, 1.0),
            shadow_depth_bias: 0.0,
            shadows_enabled: false,
            illuminance: 75_000.0 / 2.0,
            ..default()

        },
        transform: Transform::from_xyz(200.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),

        ..default()
    }).insert(Environment3d);

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
    }).insert(Environment3d);

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.4,
    });



}


fn camera_update_position(mut query: Query<(&mut Transform, &mut ActionState<CameraActions>), With<Camera>>) {
    for (mut trans, mut action) in &mut query {

        let mut rotation:Option<Quat> = None;

        if action.pressed(CameraActions::Down) {
            rotation = Some(Quat::from_rotation_x(PI / 20.0));
            action.consume(CameraActions::Down);
        }
        if action.pressed(CameraActions::Up) {
            rotation = Some(Quat::from_rotation_x(-PI / 20.0));
            action.consume(CameraActions::Up);
        }
        if action.pressed(CameraActions::Left) {
            rotation = Some(Quat::from_rotation_y(-PI / 20.0));
            action.consume(CameraActions::Left);
        }

        if action.pressed(CameraActions::Right) {
            rotation = Some(Quat::from_rotation_y(PI / 20.0));
            action.consume(CameraActions::Right);
        }


        if let Some(r) = rotation {
            let v = trans.translation.clone();
            let v2 = r.mul_vec3(v);
            let nt = Transform::from_xyz(v2.x, v2.y, v2.z).looking_at(Vec3::ZERO, Vec3::Y);

            trans.translation = nt.translation;
            trans.rotation = nt.rotation;
        }

        if action.pressed(CameraActions::Reset) {
            let nt = Transform::from_xyz(0.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y);

            trans.translation = nt.translation;
            trans.rotation = nt.rotation;
        }

    }
}