pub mod state;

use std::f32::consts::PI;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::app::{App, Plugin, Update};
use bevy::camera::Hdr;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::post_process::bloom::{Bloom, BloomPrefilter};
use bevy::prelude::{default, in_state, Assets, Camera, Camera3d, ClearColorConfig, Color, Commands, Component, DirectionalLight, Entity, GamepadButton, GlobalAmbientLight, IntoScheduleConfigs, MaterialPlugin, Mesh, MessageWriter, OnEnter, OnExit, Quat, Query, Res, ResMut, Transform, Vec2, Vec3, With};
use leafwing_input_manager::prelude::{ActionState, InputMap};
use crate::actions::CameraActions;
use crate::config::{AMBIENT_BRIGHTNESS, BLOOM_ENABLED, CAMERA_TILT, TILTED_CAMERA};
use crate::events::GameFlowEvent;
use crate::labels::SystemLabels;
use crate::level::Levels;
use crate::materials::background::BackgroundMaterial;
use crate::player::Player;
use crate::r#match::state::MatchState;
use crate::ship::ShipState;
use crate::state::GameState;
use crate::ui::{Environment3d, tear_down_3d_environment};

#[derive(Component)]
pub struct Match;

#[derive(Component)]
pub struct PlayerCamera;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(MatchState::default())

            .add_plugins(
                MaterialPlugin::<BackgroundMaterial>::default(),
            )

            .add_systems(
                OnEnter(GameState::InMatch),
                (match_spawn, setup_3d_environment).before(SystemLabels::UpdateWorld),
            )

            .add_systems(
                Update,
                camera_update_position
                    // camera_follow_ship
                    .run_if(in_state(GameState::InMatch)),
            )

            .add_systems(OnExit(GameState::InMatch), match_despawn)
            .add_systems(OnExit(GameState::PostMatch), tear_down_3d_environment)

            .add_systems(OnEnter(GameState::NextLevel), match_next_level)
        ;

    }
}


fn match_spawn(
    mut match_state: ResMut<MatchState>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
) {
    match_state.reset();

    for mut player in &mut players {
        player.reset_for_match();
    }
    commands.spawn(Match);
}

fn match_despawn(mut commands: Commands, matches: Query<Entity, With<Match>>) {
    for the_match in &matches {
        //info!("Despawn match {:?}", the_match);
        commands.entity(the_match).despawn();
    }
}


fn match_next_level(
    mut levels: ResMut<Levels>,
    mut game_event: MessageWriter<GameFlowEvent>
) {
    levels.next_level();
    game_event.write(GameFlowEvent::StartMatch);
}

fn setup_3d_environment(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<BackgroundMaterial>>,
) {
    // commands.spawn(Camera2dBundle::default());
    // camera

    let mut p = Vec3::new(0.0, 200.0, 0.00001);

    if TILTED_CAMERA {
        let q = Quat::from_rotation_x(CAMERA_TILT );
        p = q * p;
    }


    let mut camera = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(p.x, p.y, p.z).looking_at(Vec3::ZERO, Vec3::Y),
        // Transform::from_xyz(0.0, 0.0, -100.00001).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            clear_color: ClearColorConfig::Default,
            ..default()
        },
    ));

    if BLOOM_ENABLED {
        // Bloom only has an effect on an HDR camera.
        camera
            .insert(Hdr)
            .insert(Bloom {
                intensity: 0.1,
                prefilter: BloomPrefilter {
                    threshold: 1.50,
                    threshold_softness: 0.1,
                },
                scale: Vec2::splat(0.5),
                ..Bloom::NATURAL
            });
    }

    camera
        .insert(
            InputMap::default()
                .with(CameraActions::Reset, GamepadButton::North)
                .with(CameraActions::Down, GamepadButton::DPadDown)
                .with(CameraActions::Up, GamepadButton::DPadUp)
                .with(CameraActions::Left, GamepadButton::DPadLeft)
                .with(CameraActions::Right, GamepadButton::DPadRight),
        )
        .insert(Fxaa::default())
        .insert(Environment3d)
        .insert(PlayerCamera);

    // Directional Light
    const HALF_SIZE: f32 = 300.0;
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.7, 0.7, 0.9),
            shadow_depth_bias: 0.0,
            shadow_maps_enabled: true,
            illuminance: 75_000.0 / 2.0,
            ..default()
        },
        // Replaces the hand-rolled orthographic shadow projection of Bevy 0.9.
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
            color: Color::srgb(0.7, 0.7, 0.8),
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
            color: Color::srgb(0.7, 0.7, 0.7),
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


    // background

    /*commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Plane{ size: 400.0 })),
            material: materials.add(BackgroundMaterial {
                color1: Default::default(),
                color2: Default::default(),
                time: 0.0,
                alpha_mode: Default::default(),
            }),
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            ..default()
        })
        .insert(NotShadowReceiver)
*/
}


#[allow(dead_code)]
fn camera_follow_ship(
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
    ship_state: Res<ShipState>
) {
    for mut trans in &mut camera {
        trans.translation.x = ship_state.ship_position.x / 10.0;
    }
}


fn camera_update_position(mut query: Query<(&mut Transform, &ActionState<CameraActions>), With<Camera>>) {
    for (mut trans, action) in &mut query {
        let mut rotation: Option<Quat> = None;

        // `ActionState::consume` was removed in leafwing-input-manager 0.21;
        // `just_pressed` gives the same one-step-per-press behaviour.
        if action.just_pressed(&CameraActions::Down) {
            rotation = Some(Quat::from_rotation_x(PI / 20.0));
        }
        if action.just_pressed(&CameraActions::Up) {
            rotation = Some(Quat::from_rotation_x(-PI / 20.0));
        }
        if action.just_pressed(&CameraActions::Left) {
            rotation = Some(Quat::from_rotation_y(-PI / 20.0));
        }

        if action.just_pressed(&CameraActions::Right) {
            rotation = Some(Quat::from_rotation_y(PI / 20.0));
        }


        if let Some(r) = rotation {
            let v = trans.translation.clone();
            let v2 = r.mul_vec3(v);
            let nt = Transform::from_xyz(v2.x, v2.y, v2.z).looking_at(Vec3::ZERO, Vec3::Y);

            trans.translation = nt.translation;
            trans.rotation = nt.rotation;
        }

        if action.pressed(&CameraActions::Reset) {
            let nt = Transform::from_xyz(0.0, 200.0, 0.00001).looking_at(Vec3::ZERO, Vec3::Y);

            trans.translation = nt.translation;
            trans.rotation = nt.rotation;
        }
    }
}