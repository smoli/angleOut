use bevy::app::{App, Plugin, Update};
use bevy::gltf::Gltf;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{default, in_state, AlphaMode, Assets, Color, Commands, Component, Entity, GamepadButton, IntoScheduleConfigs, KeyCode, Mesh, Mesh3d, MessageWriter, OnExit, Quat, Query, Res, ResMut, Resource, Sphere, Time, Transform, Vec2, Vec3, With, Without};
use bevy::world_serialization::WorldAssetRoot;
use bevy_rapier3d::geometry::CollisionGroups;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, ExternalForce, RigidBody, Velocity};
use leafwing_input_manager::prelude::{ActionState, GamepadStick, InputMap};

use crate::actions::MatchActions;
use crate::ball::{ActiveBall, Ball};
use crate::config::{ARENA_HEIGHT_H, ARENA_WIDTH_H, BALL_RADIUS, COLLIDER_GROUP_BALL, COLLIDER_GROUP_PADDLE, COLLIDER_GROUP_PICKUP, GRAB_ATTRACT_RADIUS, GRAB_FORCE_MAGNITUDE, GRAB_RADIUS, PADDLE_LIFT, PADDLE_POSITION_MAX_ACCEL, PADDLE_RESTING_ROTATION, PADDLE_RESTING_X, PADDLE_RESTING_Y, PADDLE_RESTING_Z, PADDLE_ROTATION_ACCEL, PADDLE_THICKNESS, PADDLE_WIDTH_H};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::MyAssetPack;
use crate::physics::{Collidable, CollidableKind};
use crate::player::Player;
use crate::powerups::{Grabber, PowerUpData};
use crate::state::GameState;

#[derive(Component)]
struct DebugShape;

#[derive(Resource)]
pub struct ShipState {
    pub ship_position: Vec3,
    pub ship_rotation: f32,
}

#[derive(Component)]
pub struct Ship {
    pub asset_name: String,
    pub target_position: Vec3,
    pub target_rotation: f32,
    pub current_rotation: f32,
    pub current_accel: f32,
}

impl Default for Ship {
    fn default() -> Self {
        Ship {
            asset_name: "004_Ship_3".to_string(),
            target_position: Default::default(),
            target_rotation: 0.0,
            current_rotation: 0.0,
            current_accel: 0.0,
        }
    }
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ShipState {
                ship_position: Default::default(),
                ship_rotation: 0.0,
            })

            .add_systems(
                Update,
                (
                    ship_spawn,
                    ship_articulate,
                    ship_update_position,
                    ship_launch_ball,
                    ship_grab_ball,
                    // ship_setup_debug_grab_distances,
                )
                    .in_set(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::InMatch)),
            )

            .add_systems(
                Update,
                (ship_articulate, ship_update_position)
                    .in_set(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::PostMatch)),
            )
            .add_systems(OnExit(GameState::PostMatch), ship_despawn)
        ;
    }
}

fn ship_spawn(
    mut commands: Commands,
    my: Res<MyAssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
    empties: Query<(Entity, &Ship), With<RequestTag>>,
) {

    if let Some(gltf) = assets_gltf.get(&my.0) {

    for (entity, _ship) in &empties {
        commands.entity(entity)
            .remove::<RequestTag>()
            .insert(WorldAssetRoot(gltf.named_scenes["004_Ship_3"].clone()))
            .insert(
                InputMap::default()
                    .with_dual_axis(MatchActions::ArticulateLeft, GamepadStick::LEFT)
                    .with_dual_axis(MatchActions::ArticulateRight, GamepadStick::RIGHT)
                    .with(MatchActions::ArticulateUp, GamepadButton::RightTrigger2)
                    .with(MatchActions::ArticulateDown, GamepadButton::LeftTrigger2)

                    .with(MatchActions::SpawnOrLaunchBall, GamepadButton::RightTrigger)
                    .with(MatchActions::GrabTheBall, GamepadButton::LeftTrigger)

                    .with(MatchActions::SpawnOrLaunchBall, KeyCode::Space),
            )
            .insert(Transform::from_xyz(PADDLE_RESTING_X, PADDLE_RESTING_Y, PADDLE_RESTING_Z))
            // The paddle is driven by writing `Transform` directly. Without a rigid body it is a
            // static collider with no velocity, so a sideways sweep just depenetrates the ball a
            // little each step and drags it along. As a kinematic position based body, rapier
            // derives the paddle's velocity from the transform delta and pushes the ball away.
            .insert(RigidBody::KinematicPositionBased)
            .insert(Collider::round_cuboid(PADDLE_WIDTH_H - PADDLE_THICKNESS * 0.15, PADDLE_THICKNESS * 0.25, PADDLE_THICKNESS * 0.35, PADDLE_THICKNESS * 0.15))
            .insert(CollisionGroups::new(COLLIDER_GROUP_PADDLE, COLLIDER_GROUP_BALL | COLLIDER_GROUP_PICKUP))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Collidable {
                kind: CollidableKind::Ship
            })
        ;
    }
    }
}

fn ship_despawn(
    mut commands: Commands,
    ships: Query<Entity, With<Ship>>,
) {
    for ship in &ships {
        //info!("Despawn ship {:?}", ship);
        commands.entity(ship)
            .despawn();
    }
}

fn ship_articulate(mut query: Query<(&ActionState<MatchActions>, &mut Ship)>) {
    for (action_state, mut ship) in &mut query {
        let axis_pair_l = action_state.clamped_axis_pair(&MatchActions::ArticulateLeft);
        let axis_pair_r = action_state.clamped_axis_pair(&MatchActions::ArticulateRight);

        // Analog triggers, so read the button value rather than the pressed flag.
        let up = action_state.button_value(&MatchActions::ArticulateUp);
        let down = action_state.button_value(&MatchActions::ArticulateDown);

        if axis_pair_l == Vec2::ZERO && axis_pair_r == Vec2::ZERO && up == 0.0 && down == 0.0
        {
            ship.target_position = Vec3::new(PADDLE_RESTING_X, PADDLE_RESTING_Y, PADDLE_RESTING_Z);
            ship.target_rotation = PADDLE_RESTING_ROTATION;
            continue;
        }

        // Rotation
        let d = Vec2::new(-1.0, axis_pair_l.y) - Vec2::new(1.0, axis_pair_r.y);

        let mut a = d.perp().angle_to(Vec2::new(0.0, -1.0));
        if a.abs() < 0.1 { a = PADDLE_RESTING_ROTATION }

        ship.target_rotation = a;


        // Translation
        let comp = (axis_pair_l + axis_pair_r) * 0.75;

        let tx = if comp.length() < 0.2 {
            PADDLE_RESTING_X
        } else {
            comp.x * (ARENA_WIDTH_H - PADDLE_WIDTH_H)
        };

        let tz = ARENA_HEIGHT_H - comp.y * PADDLE_LIFT;



        // Elevation
        let t_up = up.ceil();
        let t_down = down.ceil() * -1.0;

        let ty = (t_up + t_down) * 30.0;

        let new_tp = Vec3::new(tx, PADDLE_RESTING_Y + ty, tz);

        ship.target_position = new_tp;
    }
}

fn ship_update_position(time: Res<Time>, mut ship_state: ResMut<ShipState>, mut query: Query<(&mut Transform, &mut Ship)>) {
    for (mut trans, mut ship) in &mut query {
        let dp = ship.target_position - trans.translation;

        let mut tp: Vec3 = ship.target_position;
        if dp.length() > 0.1 {
            /*            ship.current_accel += time.delta_secs() * PADDLE_POSITION_ACCEL_ACCEL;
                        if ship.current_accel > PADDLE_POSITION_MAX_ACCEL {
                            ship.current_accel = PADDLE_POSITION_MAX_ACCEL
                        }
                        tp = trans.translation + dp * time.delta_secs() * ship.current_accel;
            */
            tp = trans.translation + dp * time.delta_secs() * PADDLE_POSITION_MAX_ACCEL;
        }

        /*
                if dp.length() < 5.0 {
                    ship.current_accel = 0.0;
                    //info!("Position reached")
                }

                let nx = tp.x.clamp(PADDLE_WIDTH_H - ARENA_WIDTH_H, ARENA_WIDTH_H - PADDLE_WIDTH_H);

                if nx != tp.x {
                    ship.current_accel = 0.0;
                    //info!("Position reached");
                    tp.x = nx;
                }*/

        tp.x = tp.x.clamp(PADDLE_WIDTH_H - ARENA_WIDTH_H, ARENA_WIDTH_H - PADDLE_WIDTH_H);

        trans.translation = Vec3::new(tp.x, tp.y, tp.z);

        let dr = ship.target_rotation - ship.current_rotation;

        let mut a = ship.target_rotation;
        if dr.abs() > 0.001 {
            a = ship.current_rotation + dr * time.delta_secs() * PADDLE_ROTATION_ACCEL;
        }
        ship.current_rotation = a;
        trans.rotation = Quat::from_rotation_y(-a);

        ship_state.ship_position = trans.translation.clone();
        ship_state.ship_rotation = ship.current_rotation;
    }
}

fn ship_launch_ball(
    players: Query<&Player>,
    query: Query<&ActionState<MatchActions>, With<Ship>>,
    mut events: MessageWriter<MatchEvent>,
) {
    let Ok(player) = players.single() else { return; };

    for action in &query {
        // `ActionState::consume` was removed in leafwing-input-manager 0.21;
        // `just_pressed` gives the same fire-once-per-press behaviour.
        if action.just_pressed(&MatchActions::SpawnOrLaunchBall) {
            if player.balls_carried > 0 || player.balls_grabbed > 0 {
                //info!("Ball launch requested by operator");
                events.write(MatchEvent::BallLaunched);
            } else {
                //info!("Ball spawn requested by operator");
                events.write(MatchEvent::BallSpawned);
            }
        }
    }
}


#[allow(dead_code)]
fn ship_setup_debug_grab_distances(
    mut commands: Commands,
    ships: Query<Entity, (With<Ship>, Without<DebugShape>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ship in &ships {
        let s1 = commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Sphere::new(GRAB_RADIUS)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.0, 1.0, 0.2),
                alpha_mode: AlphaMode::Blend,
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, -PADDLE_THICKNESS * 0.7 - BALL_RADIUS),
        )).id();

        let s2 = commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Sphere::new(GRAB_ATTRACT_RADIUS)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 1.0, 1.0, 0.2),
                perceptual_roughness: 1.0,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, -PADDLE_THICKNESS * 0.7 - BALL_RADIUS),
        )).id();

        commands.entity(ship)
            .add_children(&[s1, s2])
            .insert(DebugShape);
    }
}

fn ship_grab_ball(
    mut commands: Commands,
    mut players: Query<(&mut Grabber, &Player), (Without<Ball>, Without<Ship>)>,
    ship: Query<(&ActionState<MatchActions>, &Transform), (With<Ship>, Without<Ball>)>,
    mut balls: Query<(Entity, &mut Transform, &mut ExternalForce, &mut Velocity), (With<Ball>, Without<Ship>)>,
    mut events: MessageWriter<MatchEvent>,
) {
    if let Ok((mut grabber, player)) = players.single_mut() {
        if player.balls_carried > 0 {
            return;
        }
        if !grabber.available() {
            return;
        }

        for (action, ship_trans) in &ship {
            if action.pressed(&MatchActions::GrabTheBall) {
                for (ball, mut ball_trans, mut ball_force, mut ball_velo) in &mut balls {
                    let target = ship_trans.translation + Vec3::new(0.0, 0.0, -PADDLE_THICKNESS * 0.7 - BALL_RADIUS);
                    let v = target - ball_trans.translation;
                    let d = v.length();
                    if d < GRAB_ATTRACT_RADIUS {
                        if d < GRAB_RADIUS {
                            commands.entity(ball)
                                .remove::<ActiveBall>();
                            events.write(MatchEvent::BallGrabbed);
                            ball_trans.translation = target;
                            ball_velo.linear = Vec3::ZERO;
                            grabber.use_one();
                            //info!("{} grabs left", grabber.grabs);
                        } else {
                            ball_force.force += v.normalize() * GRAB_FORCE_MAGNITUDE;
                        }
                    }
                }
            }
        }
    }
}
