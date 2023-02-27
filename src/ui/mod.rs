use bevy::app::{App, Plugin};
use bevy::log::info;
use bevy::prelude::{AmbientLight, Camera3dBundle, Commands, Component, Color, default, DespawnRecursiveExt, DirectionalLight, DirectionalLightBundle, Entity, OrthographicProjection, Query, SystemSet, Transform, Vec3, With};
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::prelude::InputManagerPlugin;
use crate::state::GameState;

mod start;
mod stats;
mod game;
mod post_match;


#[derive(Component)]
pub struct Environment3d;


#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum UIAction {
    SelectDown,
    SelectUp,
    ActivateSelection,
}


pub enum UIEvents {
    SelectionChange,
    SelectionActivated(u8)
}

pub struct UI;

impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app
            .add_event::<UIEvents>()
            .add_plugin(InputManagerPlugin::<UIAction>::default())

            .add_plugin(start::UIStartPlugin)
            .add_plugin(game::UIGamePlugin)
            .add_plugin(stats::UIStatsPlugin)
            .add_plugin(post_match::PostMatchUIPlugin)


            .add_system_set(
                SystemSet::on_enter(GameState::Start)
                    .with_system(setup_3d_environment)
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Start)
                    .with_system(tear_down_3d_environment)
            )
            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(setup_3d_environment)
            )
            .add_system_set(
                SystemSet::on_exit(GameState::InGame)
                    .with_system(tear_down_3d_environment)
            )

        ;
    }
}

pub fn tear_down_3d_environment(
    mut commands: Commands,
    env: Query<Entity, With<Environment3d>>
) {
    for e in &env {
        //info!("Teardown 3d");
        commands.entity(e)
            .despawn_recursive();
    }
}

fn setup_3d_environment(
    mut commands: Commands,
) {
    // commands.spawn(Camera2dBundle::default());
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-30.0, 20.0, -30.00001).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(Environment3d);

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
    })
        .insert(Environment3d);

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
