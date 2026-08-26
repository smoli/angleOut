use bevy::app::{PostUpdate, Update};
use bevy::prelude::{default, in_state, warn, App, AssetServer, Commands, Component, Entity, IntoScheduleConfigs, MessageReader, MessageWriter, OnExit, Plugin, Quat, Query, Res, Time, Transform, Vec3, Visibility, With, Without};
use bevy::world_serialization::WorldAssetRoot;
use crate::state::GameState;
use std::f32::consts::TAU;
use bevy_rapier3d::prelude::{ActiveEvents, Ccd, CoefficientCombineRule, Collider, ColliderMassProperties, CollisionGroups, Damping, ExternalForce, ExternalImpulse, Friction, GravityScale, LockedAxes, MassProperties, Restitution, Sleeping, Velocity};
use bevy_rapier3d::dynamics::RigidBody;
use crate::config::{BALL_RADIUS, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_NONE, COLLIDER_GROUP_PADDLE, MAX_BALL_SPEED, MAX_RESTITUTION, MIN_BALL_SPEED, PADDLE_THICKNESS};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::physics::{Collidable, CollidableKind, CollisionEventHandling, CollisionInfo, CollisionTag};
use crate::ship::ShipState;

#[derive(Component)]
pub struct Ball {
    pub asset_name: String,
}

impl Default for Ball {
    fn default() -> Self {
        Ball { asset_name: "ship3_003.glb#Scene0".to_string() }
    }
}


#[derive(Component)]
pub struct ActiveBall;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app

            .add_systems(
                Update,
                ball_clear_external_forces
                    .before(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::InMatch)),
            )

            .add_systems(
                Update,
                (
                    ball_spawn,
                    ball_spin,
                    ball_update_inactive,
                    ball_correct_too_low_z,
                    ball_inactive_handle_events,
                )
                    .in_set(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::InMatch)),
            )

            .add_systems(PostUpdate, ball_handle_collisions.in_set(CollisionEventHandling))

/*            .add_systems(
                Update,
                ball_limit_velocity
                    .after(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::InMatch)),
            )
*/
            .add_systems(OnExit(GameState::PostMatch), ball_despawn)
        ;
    }
}

pub fn ball_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    empties: Query<(Entity, &Ball), With<RequestTag>>)
{
    for (entity, ball) in &empties {
        commands.entity(entity)
            .remove::<RequestTag>()
            .insert(WorldAssetRoot(asset_server.load(ball.asset_name.clone())))
            .insert(Visibility::Visible)
            .insert(Transform::default())
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(Collider::ball(BALL_RADIUS))
            .insert(Restitution {
                coefficient: MAX_RESTITUTION,
                combine_rule: CoefficientCombineRule::Max,
            })
            .insert(Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            })
            .insert(Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            })
            .insert(Sleeping::disabled())
            .insert(ColliderMassProperties::Density(20.0))
            .insert(ColliderMassProperties::MassProperties(MassProperties {
                mass: 1.0,
                ..default()
            }))
            .insert(Velocity::default())
            .insert(ExternalImpulse::default())
            .insert(ExternalForce::default())
            .insert(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED)
            .insert(CollisionGroups::new(COLLIDER_GROUP_BALL, COLLIDER_GROUP_NONE))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Ccd::enabled())
            .insert(Collidable {
                kind: CollidableKind::Ball
            })
        ;
    }
}

fn ball_despawn(
    mut commands: Commands,
    balls: Query<Entity, With<Ball>>,
) {
    for ball in &balls {
        //info!("despawn ball {:?}", ball);
        commands.entity(ball)
            .despawn();
    }
}


fn ball_spin(
    timer: Res<Time>,
    mut ball: Query<&mut Transform, With<ActiveBall>>) {
    for mut trans in &mut ball {
        trans.rotate_y(1.0 * TAU * timer.delta_secs());
    }
}

fn ball_update_inactive(
    ship_state: Res<ShipState>,
    mut query: Query<&mut Transform, (Without<ActiveBall>, With<Ball>)>)
{
    for mut trans in &mut query {
        trans.translation = ship_state.ship_position.clone() + Vec3::new(0.0, 0.0, -PADDLE_THICKNESS * 0.7 - BALL_RADIUS);
        // velo.linear = Vec3::ZERO;
        // impulse.impulse = Vec3::ZERO;
    }
}

fn ball_clear_external_forces(
    mut balls: Query<&mut ExternalForce, With<Ball>>
) {
    for mut force in &mut balls {
        force.force = Vec3::ZERO;
    }
}

pub fn compute_launch_impulse(angle: f32, value: f32) -> Vec3 {
    //                                       Z-Axis: negative is up
    let imp = Vec3::new(0.0, 0.0, -value);
    Quat::from_rotation_y(-angle).mul_vec3(imp)
}


fn ball_inactive_handle_events(
    mut commands: Commands,
    mut events: MessageReader<MatchEvent>,
    ship_state: Res<ShipState>,
    mut balls: Query<(Entity, &mut Velocity, &mut CollisionGroups), (Without<ActiveBall>, With<Ball>)>)
{
    for (ball, mut velo, mut col) in &mut balls {
        for ev in events.read() {
            match ev {
                MatchEvent::BallSpawned => {}
                MatchEvent::BallLaunched => {
                    velo.linear = compute_launch_impulse(ship_state.ship_rotation, MIN_BALL_SPEED);
                    commands.entity(ball)
                        .insert(ActiveBall);
                    col.filters = col.filters | COLLIDER_GROUP_PADDLE | COLLIDER_GROUP_BLOCK;
                }

                _ => {}
            }
        }
    }
}

#[allow(dead_code)]
fn ball_limit_velocity(mut query: Query<(&mut Velocity, &ExternalForce), With<ActiveBall>>) {
    for (mut velo, ext_force) in &mut query {
        let v = velo.linear.length();

        if v == 0.0 {
            //info!("No speed");
            continue;
        }

        if ext_force.force.length() != 0.0 {
            continue;
        }

        if v > MAX_BALL_SPEED {
            velo.linear = velo.linear * MAX_BALL_SPEED / v;
        } else if v < MIN_BALL_SPEED {
            velo.linear = velo.linear * MIN_BALL_SPEED / v;
        }

        if velo.linear.y != 0.0 {
            warn!("It wants to break free!");
            velo.linear.y = 0.0
        }
    }
}

fn ball_correct_too_low_z(mut query: Query<&mut Velocity, With<ActiveBall>>) {
    for mut velo in &mut query {
        let v = velo.linear.length();

        if velo.linear.z.abs() < 30.0 {
            //info!("Correcting Z velocity for more fun!");

            velo.linear.z = 35.0 * velo.linear.z.signum();

            velo.linear = velo.linear.normalize() * v;
        }
    }
}


fn ball_handle_collisions(
    mut commands: Commands,
    mut balls: Query<(Entity, &mut Velocity), (With<ActiveBall>, With<CollisionTag>)>,
    mut events: MessageWriter<MatchEvent>,
    collisions: Res<CollisionInfo>,
) {
    for (ball, mut velo) in &mut balls {
        let mut correct_ball_trans = false;

        if let Some(collision) = collisions.collisions.get(&ball) {
            for collision in collision {
                match collision.other {
                    CollidableKind::Ship => {
                        correct_ball_trans = true;
                 /*       ext_imp.impulse = compute_launch_impulse(
                            ship_state.ship_rotation, PADDLE_BOUNCE_IMPULSE,
                        );*/

                        commands.entity(ball)
                            .remove::<CollisionTag>();
                        events.write(MatchEvent::BounceOffPaddle);
                    }

                    CollidableKind::Wall => {
                        events.write(MatchEvent::BounceOffWall);
                    }

                    CollidableKind::DeathTrigger => {
                        events.write(MatchEvent::BallLost);

                        commands.entity(ball)
                            .despawn();
                    }

                    CollidableKind::Block => {
                        correct_ball_trans = true;
                    }

                    _ => {}
                }
            }
        }

        if correct_ball_trans {
            let v = velo.linear.length();

            if velo.linear.z.abs() < 1.0 {
                //info!("Correcting Z velocity for more fun!");

                velo.linear.z = 3.0 * velo.linear.z.signum();

                velo.linear = velo.linear.normalize() * v;
            }


            let v = velo.linear.length();
            //info!("Exit speed {}", v);
            if v > MAX_BALL_SPEED {
                velo.linear = velo.linear * MAX_BALL_SPEED / v;
            } else if v < MIN_BALL_SPEED {
                velo.linear = velo.linear * MIN_BALL_SPEED / v;
                //info!("Prevented too slow of a ball for more fun! New speed {}", velo.linear.length());
            }
        }
    }
}