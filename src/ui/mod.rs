use bevy::app::{App, Plugin};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::{default, Camera3d, Color, Commands, Component, DirectionalLight, Entity, GlobalAmbientLight, Message, OnEnter, OnExit, Query, Reflect, Transform, Vec3, With};
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::prelude::InputManagerPlugin;
use crate::config::AMBIENT_BRIGHTNESS;
use crate::state::GameState;

mod start;
pub mod stats;
mod game;
mod post_match;


#[derive(Component)]
pub struct Environment3d;


#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum UIAction {
    SelectDown,
    SelectUp,
    ActivateSelection,
}


#[derive(Message)]
pub enum UIEvents {
    SelectionChange,
    SelectionActivated(u8)
}

pub struct UI;

impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app
            .add_message::<UIEvents>()
            .add_plugins(InputManagerPlugin::<UIAction>::default())

            .add_plugins(start::UIStartPlugin)
            .add_plugins(game::UIGamePlugin)
            .add_plugins(stats::UIStatsPlugin)
            .add_plugins(post_match::PostMatchUIPlugin)


            .add_systems(OnEnter(GameState::Start), setup_3d_environment)
            .add_systems(OnExit(GameState::Start), tear_down_3d_environment)
            .add_systems(OnEnter(GameState::InGame), setup_3d_environment)
            .add_systems(OnExit(GameState::InGame), tear_down_3d_environment)

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
            .despawn();
    }
}

fn setup_3d_environment(
    mut commands: Commands,
) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-30.0, 20.0, -30.00001).looking_at(Vec3::ZERO, Vec3::Y),
        Environment3d,
    ));

    // Directional Light
    const HALF_SIZE: f32 = 300.0;
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.7, 0.7, 1.0),
            shadow_depth_bias: 0.0,
            shadow_maps_enabled: true,
            illuminance: 75_000.0 / 2.0,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 2.0 * HALF_SIZE,
            ..default()
        }
            .build(),
        Transform::from_xyz(200.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),
        Environment3d,
    ));

    // Directional Light
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.7, 0.7, 1.0),
            shadow_depth_bias: 0.0,
            shadow_maps_enabled: false,
            illuminance: 75_000.0 / 2.0,
            ..default()
        },
        Transform::from_xyz(200.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),
        Environment3d,
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.7, 0.7, 1.0),
            shadow_depth_bias: 0.0,
            shadow_maps_enabled: false,
            illuminance: 5_000.0,
            ..default()
        },
        Transform::from_xyz(-200.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y),
        Environment3d,
    ));

    // ambient light
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });
}
