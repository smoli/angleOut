use bevy::app::App;
use bevy::asset::Handle;
use bevy::prelude::{AssetServer, Commands, Component, DespawnRecursiveExt, Entity, EventWriter, GamepadButtonType, info, IntoSystemDescriptor, KeyCode, Plugin, Quat, Query, Res, ResMut, Resource, SystemSet, Time, Transform, TransformBundle, Vec2, Vec3, With, Without};
use bevy::scene::{Scene, SceneBundle};
use bevy::utils::default;
use bevy_rapier3d::geometry::CollisionGroups;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, ExternalForce};
use leafwing_input_manager::axislike::DualAxisData;
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::{ActionState, DualAxis, InputMap};
use crate::actions::MatchActions;
use crate::ball::{ActiveBall, Ball};
use crate::config::{ARENA_HEIGHT_H, ARENA_WIDTH_H, BALL_RADIUS, COLLIDER_GROUP_BALL, COLLIDER_GROUP_PADDLE, GRAB_ATTRACT_RADIUS, GRAB_FORCE_MAGNITUDE, GRAB_RADIUS, PADDLE_LIFT, PADDLE_POSITION_ACCEL_ACCEL, PADDLE_POSITION_MAX_ACCEL, PADDLE_RESTING_ROTATION, PADDLE_RESTING_X, PADDLE_RESTING_Y, PADDLE_RESTING_Z, PADDLE_ROTATION_ACCEL, PADDLE_THICKNESS, PADDLE_WIDTH_H};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::physics::{Collidable, CollidableKind};
use crate::player::{Player, PowerUp};
use crate::state::GameState;

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
    pub current_accel: f32
}

impl Default for Ship {
    fn default() -> Self {
        Ship {
            asset_name: "ship3_003.glb#Scene4".to_string(),
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

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(ship_spawn.label(SystemLabels::UpdateWorld))
                    .with_system(ship_articulate.label(SystemLabels::UpdateWorld))
                    .with_system(ship_update_position.label(SystemLabels::UpdateWorld))
                    .with_system(ship_launch_ball.label(SystemLabels::UpdateWorld))
                    .with_system(ship_grab_ball.label(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatchLoose)
                    .with_system(ship_despawn)
            )
        ;
    }
}

pub fn ship_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    empties: Query<(Entity, &Ship), With<RequestTag>>,
) {
    for (entity, ship) in &empties {
        commands.entity(entity)
            .remove::<RequestTag>()
            .insert(SceneBundle {
                scene: asset_server.load(ship.asset_name.as_str()),
                ..default()
            })
            .insert(InputManagerBundle::<MatchActions> {
                action_state: ActionState::default(),
                input_map: InputMap::default()
                    .insert(DualAxis::left_stick(), MatchActions::ArticulateLeft)
                    .insert(DualAxis::right_stick(), MatchActions::ArticulateRight)
                    .insert(GamepadButtonType::RightTrigger, MatchActions::SpawnOrLaunchBall)
                    .insert(GamepadButtonType::RightTrigger2, MatchActions::SpawnOrLaunchBall)
                    .insert(KeyCode::Space, MatchActions::SpawnOrLaunchBall)
                    .insert(GamepadButtonType::LeftTrigger2, MatchActions::GrabTheBall)
                    .build(),
            })
            .insert(TransformBundle::from(Transform::from_xyz(PADDLE_RESTING_X, PADDLE_RESTING_Y, PADDLE_RESTING_Z)))
            .insert(Collider::round_cuboid(PADDLE_WIDTH_H - PADDLE_THICKNESS * 0.15, PADDLE_THICKNESS, PADDLE_THICKNESS * 0.35, PADDLE_THICKNESS * 0.15))
            .insert(CollisionGroups::new(COLLIDER_GROUP_PADDLE, COLLIDER_GROUP_BALL))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Collidable {
                kind: CollidableKind::Ship
            });
    }
}

fn ship_despawn(
    mut commands: Commands,
    ships: Query<Entity, With<Ship>>
) {
    for ship in &ships {
        commands.entity(ship)
            .despawn_recursive();
    }
}

fn ship_articulate(mut query: Query<(&ActionState<MatchActions>, &mut Ship)>) {
    for (action_state, mut ship) in &mut query {
        if !action_state.pressed(MatchActions::ArticulateLeft) && !action_state.pressed(MatchActions::ArticulateRight) {
            ship.target_position = Vec3::new(PADDLE_RESTING_X, PADDLE_RESTING_Y, PADDLE_RESTING_Z);
            ship.target_rotation = PADDLE_RESTING_ROTATION;
            return;
        }

        let axis_pair_l: DualAxisData = action_state.clamped_axis_pair(MatchActions::ArticulateLeft).unwrap();
        let axis_pair_r: DualAxisData = action_state.clamped_axis_pair(MatchActions::ArticulateRight).unwrap();

        // Rotation
        let d = Vec2::new(-1.0, axis_pair_l.y()) - Vec2::new(1.0, axis_pair_r.y());

        let mut a = d.perp().angle_between(Vec2::new(0.0, -1.0));
        if a.abs() < 0.1 { a = PADDLE_RESTING_ROTATION }

        ship.target_rotation = a;


        // Translation
        let comp = (axis_pair_l.xy() + axis_pair_r.xy()) * 0.75;

        let tx = if comp.length() < 0.2 {
            PADDLE_RESTING_X
        } else {
            comp.x * (ARENA_WIDTH_H - PADDLE_WIDTH_H)
        };

        let tz = ARENA_HEIGHT_H - comp.y * PADDLE_LIFT;

        let new_tp = Vec3::new(tx, PADDLE_RESTING_Y, tz);

        ship.target_position = new_tp;
    }
}

fn ship_update_position(time: Res<Time>, mut ship_state: ResMut<ShipState>, mut query: Query<(&mut Transform, &mut Ship)>) {
    for (mut trans, mut ship) in &mut query {
        let dp = ship.target_position - trans.translation;

        let mut tp: Vec3 = ship.target_position;
        if dp.length() > 0.1 {
/*            ship.current_accel += time.delta_seconds() * PADDLE_POSITION_ACCEL_ACCEL;
            if ship.current_accel > PADDLE_POSITION_MAX_ACCEL {
                ship.current_accel = PADDLE_POSITION_MAX_ACCEL
            }
            tp = trans.translation + dp * time.delta_seconds() * ship.current_accel;
*/
            tp = trans.translation + dp * time.delta_seconds() * PADDLE_POSITION_MAX_ACCEL;
        }

/*
        if dp.length() < 5.0 {
            ship.current_accel = 0.0;
            info!("Position reached")
        }

        let nx = tp.x.clamp(PADDLE_WIDTH_H - ARENA_WIDTH_H, ARENA_WIDTH_H - PADDLE_WIDTH_H);

        if nx != tp.x {
            ship.current_accel = 0.0;
            info!("Position reached");
            tp.x = nx;
        }*/

        tp.x = tp.x.clamp(PADDLE_WIDTH_H - ARENA_WIDTH_H, ARENA_WIDTH_H - PADDLE_WIDTH_H);

        trans.translation = Vec3::new(tp.x, tp.y, tp.z);

        let dr = ship.target_rotation - ship.current_rotation;

        let mut a = ship.target_rotation;
        if dr.abs() > 0.001 {
            a = ship.current_rotation + dr * time.delta_seconds() * PADDLE_ROTATION_ACCEL;
        }
        ship.current_rotation = a;
        trans.rotation = Quat::from_rotation_y(-a);

        ship_state.ship_position = trans.translation.clone();
        ship_state.ship_rotation = ship.current_rotation;
    }
}

fn ship_launch_ball(
    player: Res<Player>,
    mut query: Query<&mut ActionState<MatchActions>, With<Ship>>,
    mut events: EventWriter<MatchEvent>,
) {
    for mut action in &mut query {
        if action.pressed(MatchActions::SpawnOrLaunchBall) {
            action.consume(MatchActions::SpawnOrLaunchBall);

            if player.balls_spawned > 0 || player.balls_grabbed > 0 {
                info!("Ball launch requested by operator");
                events.send(MatchEvent::BallLaunched);
            } else {
                info!("Ball spawn requested by operator");
                events.send(MatchEvent::BallSpawned);
            }

        }
    }
}

fn ship_grab_ball(
    mut commands: Commands,
    player: Res<Player>,
    ship: Query<(&ActionState<MatchActions>, &Transform), (With<Ship>, Without<Ball>)>,
    mut balls: Query<(Entity, &mut Transform, &mut ExternalForce), (With<Ball>, Without<Ship>)>,
    mut events: EventWriter<MatchEvent>,
) {
    if !player.power_ups.contains(&PowerUp::Grabber) {
        return;
    }

    if player.balls_grabbed > 0 {
        return;
    }
    for (action, ship_trans) in &ship {
        if action.pressed(MatchActions::GrabTheBall) {

            for (ball, mut ball_trans, mut ball_force) in &mut balls {
                let target = ship_trans.translation + Vec3::new(0.0, 0.0, -PADDLE_THICKNESS * 0.7 - BALL_RADIUS);
                let v = target - ball_trans.translation;
                let d = v.length();
                if d < GRAB_ATTRACT_RADIUS {
                    info!("Distance {}", d);
                    if d < GRAB_RADIUS {
                        info!("Grabbing");
                        commands.entity(ball)
                            .remove::<ActiveBall>();
                        events.send(MatchEvent::BallGrabbed);
                        ball_trans.translation = target;
                    } else {
                        info!("Pulling in");
                        ball_force.force += v.normalize() * GRAB_FORCE_MAGNITUDE;
                    }
                }
            }


        }
    }
}
