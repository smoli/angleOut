use bevy::prelude::{App, AssetServer, Commands, Component, Entity, EventReader, IntoSystemDescriptor, Plugin, Quat, Query, Res, SystemSet, Time, Transform, TransformBundle, Vec3, Visibility, warn, With, Without};
use bevy::scene::SceneBundle;
use bevy::utils::default;
use crate::state::GameState;
use std::f32::consts::TAU;
use bevy::log::info;
use bevy::utils::tracing::enabled;
use bevy_rapier3d::prelude::{ActiveEvents, Ccd, Collider, ColliderMassProperties, CollisionGroups, Damping, ExternalImpulse, Friction, GravityScale, LockedAxes, MassProperties, Restitution, Velocity};
use bevy_rapier3d::dynamics::RigidBody;
use crate::config::{BALL_RADIUS, COLLIDER_GROUP_BALL, COLLIDER_GROUP_BLOCK, COLLIDER_GROUP_NONE, COLLIDER_GROUP_PADDLE, MAX_BALL_SPEED, MAX_RESTITUTION, MIN_BALL_SPEED, PADDLE_BOUNCE_IMPULSE, PADDLE_LAUNCH_IMPULSE, PADDLE_THICKNESS};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::physics::{Collidable, CollidableKind, CollisionTag};
use crate::ship::ShipState;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct ActiveBall;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(ball_spawn)
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    // .with_system(ball_spin.label(SystemLabels::UpdateWorld))
                    .with_system(ball_update_inactive.label(SystemLabels::UpdateWorld))
                    .with_system(ball_inactive_handle_events.label(SystemLabels::UpdateWorld))
                    .with_system(ball_inactive_handle_events.label(SystemLabels::UpdateWorld))
                    .with_system(ball_clamp_velocity.label(SystemLabels::UpdateWorld))
                    .with_system(ball_handle_collisions.label(SystemLabels::UpdateWorld))
            )
        ;
    }
}

fn ball_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>)
{
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("ship3_003.gltf#Scene0"),
            visibility: Visibility {
                is_visible: true
            },
            ..default()
        })
        .insert(TransformBundle::default())
        .insert(Ball)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(0.0))
        .insert(Collider::ball(BALL_RADIUS))
        .insert(Restitution::coefficient(MAX_RESTITUTION))
        .insert(Damping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        })
        .insert(Friction::coefficient(0.0))
        .insert(ColliderMassProperties::Density(20.0))
        .insert(ColliderMassProperties::MassProperties(MassProperties {
            mass: 1.0,
            ..default()

        }))
        .insert(Velocity::default())
        .insert(ExternalImpulse::default())
        .insert(LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED)
        .insert(CollisionGroups::new(COLLIDER_GROUP_BALL, COLLIDER_GROUP_NONE))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Ccd::enabled())
        .insert(Collidable {
            kind: CollidableKind::Ball
        })

    ;
}

fn ball_spin(
    timer: Res<Time>,
    mut ball: Query<&mut Transform, With<Ball>>) {
    for mut trans in &mut ball {
        trans.rotate_y(1.0 * TAU * timer.delta_seconds());
    }
}

fn ball_update_inactive(
    ship_state: Res<ShipState>,
    mut query: Query<&mut Transform, (Without<ActiveBall>, With<Ball>)>)
{
    for mut trans in &mut query {
        trans.translation = ship_state.ship_position.clone() + Vec3::new(0.0, 0.0, -PADDLE_THICKNESS * 0.5 - BALL_RADIUS);
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
    mut balls: Query<(Entity, &mut ExternalImpulse, &mut CollisionGroups), (Without<ActiveBall>, With<Ball>)>)
{
    for (ball, mut ext_imp, mut col) in &mut balls {
        for ev in events.iter() {
            match ev {
                MatchEvent::SpawnBall => {}
                MatchEvent::LaunchBall => {
                    ext_imp.impulse = compute_launch_impulse(ship_state.ship_rotation, PADDLE_LAUNCH_IMPULSE);
                    commands.entity(ball)
                        .insert(ActiveBall);
                    col.filters = col.filters | COLLIDER_GROUP_PADDLE | COLLIDER_GROUP_BLOCK;
                }
                MatchEvent::LooseBall => {}
                MatchEvent::BounceOfPaddle => {}
                MatchEvent::DestroyFoe => {}

                _ => {}
            }
        }
    }
}

fn ball_clamp_velocity(mut query: Query<(&mut Velocity, &mut Transform), With<ActiveBall>>) {
    for (mut velo, mut trans) in &mut query {
        let v = velo.linvel.length();

        if v > MAX_BALL_SPEED {
            velo.linvel = velo.linvel * MAX_BALL_SPEED / v;
        } else if v < MIN_BALL_SPEED {
            velo.linvel = velo.linvel * MIN_BALL_SPEED / v;
        }

        if trans.translation.y != 0.0 {
            trans.translation.y = 0.0;
            warn!("Correcting ball height! We're loosing that ball")
        }
    }
}


fn ball_handle_collisions(
    mut commands: Commands,
    ship_state: Res<ShipState>,
    mut balls: Query<(Entity, &mut ExternalImpulse, &CollisionTag), With<Ball>>
) {
    for (ball, mut ext_imp, collision) in &mut balls {
        match collision.other {
            CollidableKind::Ship => {

                ext_imp.impulse = compute_launch_impulse(
                    ship_state.ship_rotation, PADDLE_BOUNCE_IMPULSE
                );

                commands.entity(ball)
                    .remove::<CollisionTag>();

                info!("Applied bounce impulse");
            }

            _ => {}
        }
    }
}