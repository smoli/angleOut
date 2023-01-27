use bevy::prelude::{App, AssetServer, Commands, Component, DespawnRecursiveExt, Entity, EventReader, EventWriter, IntoSystemDescriptor, Plugin, Quat, Query, Res, SystemSet, Time, Transform, TransformBundle, Vec3, Visibility, warn, With, Without};
use bevy::scene::SceneBundle;
use bevy::utils::default;
use crate::state::GameState;
use std::f32::consts::TAU;
use bevy::log::info;
use bevy_rapier3d::prelude::{ActiveEvents, Ccd, CoefficientCombineRule, Collider, ColliderMassProperties, CollisionGroups, Damping, ExternalForce, ExternalImpulse, Friction, GravityScale, LockedAxes, MassProperties, Restitution, Sleeping, Velocity};
use bevy_rapier3d::dynamics::RigidBody;
use crate::config::{BALL_RADIUS, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_NONE, COLLIDER_GROUP_PADDLE, MAX_BALL_SPEED, MAX_RESTITUTION, MIN_BALL_SPEED, PADDLE_BOUNCE_IMPULSE, PADDLE_LAUNCH_IMPULSE, PADDLE_THICKNESS};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::RequestTag;
use crate::physics::{Collidable, CollidableKind, CollisionTag};
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

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(ball_clear_external_forces.before(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(ball_spawn.label(SystemLabels::UpdateWorld))
                    .with_system(ball_spin.label(SystemLabels::UpdateWorld))
                    .with_system(ball_update_inactive.label(SystemLabels::UpdateWorld))
                    // .with_system(ball_correct_too_low_z.label(SystemLabels::UpdateWorld))
                    .with_system(ball_handle_collisions.label(SystemLabels::UpdateWorld))
                    .with_system(ball_inactive_handle_events.label(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(ball_limit_velocity
                        .after(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatch)
                    .with_system(ball_despawn)
            )
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
            .insert(SceneBundle {
                scene: asset_server.load(ball.asset_name.as_str()),
                visibility: Visibility {
                    is_visible: true
                },
                ..default()
            })
            .insert(TransformBundle::default())
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
    balls: Query<Entity, With<Ball>>
) {
    for ball in &balls {
        commands.entity(ball)
            .despawn_recursive();
    }
}


fn ball_spin(
    timer: Res<Time>,
    mut ball: Query<&mut Transform, With<ActiveBall>>) {
    for mut trans in &mut ball {
        trans.rotate_y(1.0 * TAU * timer.delta_seconds());
    }
}

fn ball_update_inactive(
    ship_state: Res<ShipState>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut ExternalImpulse), (Without<ActiveBall>, With<Ball>)>)
{
    for (mut trans, mut velo, mut impulse) in &mut query {
        trans.translation = ship_state.ship_position.clone() + Vec3::new(0.0, 0.0, -PADDLE_THICKNESS * 0.7 - BALL_RADIUS);
        velo.linvel = Vec3::ZERO;
        impulse.impulse = Vec3::ZERO;
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
    mut events: EventReader<MatchEvent>,
    ship_state: Res<ShipState>,
    mut balls: Query<(Entity, &mut ExternalForce, &mut CollisionGroups), (Without<ActiveBall>, With<Ball>)>)
{
    for (ball, mut ext_force, mut col) in &mut balls {
        for ev in events.iter() {
            match ev {
                MatchEvent::BallSpawned => {}
                MatchEvent::BallLaunched => {
                    ext_force.force = compute_launch_impulse(ship_state.ship_rotation, PADDLE_LAUNCH_IMPULSE);
                    commands.entity(ball)
                        .insert(ActiveBall);
                    col.filters = col.filters | COLLIDER_GROUP_PADDLE | COLLIDER_GROUP_BLOCK;
                }

                _ => {}
            }
        }
    }
}

fn ball_limit_velocity(mut query: Query<(&mut Velocity, &ExternalForce), With<ActiveBall>>) {
    for (mut velo, mut ext_force) in &mut query {
        let v = velo.linvel.length();

        if v == 0.0 {
            continue;
        }

        if ext_force.force.length() != 0.0 {
            continue;
        }

        if v > MAX_BALL_SPEED {
            velo.linvel = velo.linvel * MAX_BALL_SPEED / v;
        } else if v < MIN_BALL_SPEED {
            velo.linvel = velo.linvel * MIN_BALL_SPEED / v;
        }

        if velo.linvel.y != 0.0 {
            warn!("It wants to break free!");
            velo.linvel.y = 0.0
        }
    }
}

fn ball_correct_too_low_z(mut query: Query<&mut Velocity, With<ActiveBall>>) {
    for mut velo in &mut query {
        let v = velo.linvel.length();

        if velo.linvel.z.abs() < 1.0 {
            info!("Correcting Z velocity for more fun!");

            velo.linvel.z = 3.0 * velo.linvel.z.signum();

            velo.linvel = velo.linvel.normalize() * v;
        }
    }
}


fn ball_handle_collisions(
    mut commands: Commands,
    ship_state: Res<ShipState>,
    mut balls: Query<(Entity, &mut ExternalImpulse, &CollisionTag, &mut Velocity), With<ActiveBall>>,
    mut events: EventWriter<MatchEvent>,
) {
    for (ball, mut ext_imp, collision, mut velo) in &mut balls {
        let mut correct_ball_trans = false;

        match collision.other {
            CollidableKind::Ship => {
                correct_ball_trans = true;
                ext_imp.impulse = compute_launch_impulse(
                    ship_state.ship_rotation, PADDLE_BOUNCE_IMPULSE,
                );

                commands.entity(ball)
                    .remove::<CollisionTag>();

                info!("Applied bounce impulse");

                events.send(MatchEvent::BounceOffPaddle);
            }

            CollidableKind::Wall => {
                events.send(MatchEvent::BounceOffWall);
            }

            CollidableKind::DeathTrigger => {
                events.send(MatchEvent::BallLost);

                commands.entity(ball)
                    .despawn_recursive();
            }

            CollidableKind::Block => {
                correct_ball_trans = true;
            }

            _ => {}
        }

        if correct_ball_trans {
            let v = velo.linvel.length();

            if velo.linvel.z.abs() < 1.0 {
                info!("Correcting Z velocity for more fun!");

                velo.linvel.z = 3.0 * velo.linvel.z.signum();

                velo.linvel = velo.linvel.normalize() * v;
            }


            let v = velo.linvel.length();
            info!("Exit speed {}", v);
            if v > MAX_BALL_SPEED {
                velo.linvel = velo.linvel * MAX_BALL_SPEED / v;
            } else if v < MIN_BALL_SPEED {
                velo.linvel = velo.linvel * MIN_BALL_SPEED / v;
                info!("Prevented too slow of a ball for more fun! New speed {}", velo.linvel.length());
            }
        }
    }
}