use bevy::app::{App, Plugin, PostUpdate, Update};
use bevy::gltf::Gltf;
use bevy::prelude::{in_state, Assets, Commands, Component, Entity, IntoScheduleConfigs, MessageReader, MessageWriter, OnExit, Query, Res, ResMut, Time, Transform, Vec3, With};
use bevy::world_serialization::WorldAssetRoot;
use bevy_rapier3d::dynamics::GravityScale;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionGroups, RigidBody};
use serde::{Deserialize, Serialize};

use crate::config::{COLLIDER_GROUP_DEATH, COLLIDER_GROUP_PADDLE, COLLIDER_GROUP_PICKUP, PICKUP_GENERIC_SCENE, PICKUP_SPEED};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::{Levels, RequestTag};
use crate::MyAssetPack;
use crate::physics::{Collidable, CollidableKind, CollisionEventHandling, CollisionInfo, CollisionTag};
use crate::r#match::state::MatchState;
use crate::state::GameState;

pub struct PickupsPlugin;

impl Plugin for PickupsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (pickup_spawn, pickup_update, pickup_spawn_globals_on_event)
                    .in_set(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::InMatch)),
            )

            .add_systems(PostUpdate, pickup_handle_collisions.in_set(CollisionEventHandling))

            .add_systems(OnExit(GameState::PostMatch), pickup_despawn_all)
        ;
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PickupType {
    MoreBalls(i32),
    Grabber(i16),
}

#[derive(Component, Debug)]
pub struct Pickup {
    pub spawn_position: Vec3,
    pub pickup_type: PickupType,
}


#[derive(Component)]
pub struct Fall {
    pub dir: Vec3,
}


fn pickup_spawn_globals_on_event(
    mut commands: Commands,
    mut events: MessageReader<MatchEvent>,
    match_state: ResMut<MatchState>,
    levels: ResMut<Levels>,
) {
    //  let (player_entity, mut player, mut bouncer) = players.get_single_mut().unwrap();

    let level = levels.get_current_level().unwrap();

    for ev in events.read() {
        match ev {
            MatchEvent::BlockHit(p, _block_type, _behaviour) => {
                if let Some(pickup_type) = level.pickup_at(match_state.blocks as usize) {
                    commands.spawn(Pickup {
                        spawn_position: p.clone(),
                        pickup_type: pickup_type.clone(),
                    })
                        .insert(RequestTag);
                }
            }

            _ => {}
        }
    }
}


fn pickup_despawn_all(
    mut commands: Commands,
    pickups: Query<Entity, With<Pickup>>,
) {
    for p in &pickups {
        //info!("Despan pickup {:?}", p);
        commands.entity(p)
            .despawn();
    }
}

fn pickup_spawn(
    mut commands: Commands,
    asset_pack: Res<MyAssetPack>,
    requests: Query<(Entity, &Pickup), With<RequestTag>>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(gltf) = assets_gltf.get(&asset_pack.0) {
        for (entity, pickup) in &requests {
            commands.entity(entity)
                .remove::<RequestTag>()


                .insert(WorldAssetRoot(gltf.named_scenes[PICKUP_GENERIC_SCENE].clone()))
                .insert(Transform::from_translation(pickup.spawn_position.clone()))
                .insert(Fall {
                    dir: Vec3::new(0.0, 0.0, PICKUP_SPEED)
                })
                .insert(Collider::cuboid(
                    2.0, 2.0, 2.0,
                ))

                .insert(RigidBody::Dynamic)     // FIXME: Why do I need the rigid body here?
                .insert(GravityScale(0.0))      // FIXME: This is only needed because of the rigid body.
                .insert(CollisionGroups::new(COLLIDER_GROUP_PICKUP, COLLIDER_GROUP_DEATH | COLLIDER_GROUP_PADDLE))

                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collidable {
                    kind: CollidableKind::Pickup,
                })
            ;
        }
    }
}

fn pickup_update(
    time: Res<Time>,
    mut pickups: Query<(&Fall, &mut Transform), With<Pickup>>,
) {
    for (fall, mut trans) in &mut pickups {
        trans.translation += fall.dir * time.delta_secs();
    }
}

fn pickup_handle_collisions(
    mut commands: Commands,
    pickups: Query<(Entity, &Pickup, &CollisionTag)>,
    mut events: MessageWriter<MatchEvent>,
    collisions: Res<CollisionInfo>,
) {
    for (entity, pickup, _) in &pickups {
        if let Some(collision) = collisions.collisions.get(&entity) {
            for collision in collision {
                match collision.other {
                    CollidableKind::Ship => {
                        events.write(MatchEvent::PickedUp(pickup.pickup_type));
                    }

                    CollidableKind::DeathTrigger => {}

                    _ => {}
                }
            }
        }

        //info!("Despanw pickup regardless {:?}", entity);
        commands.entity(entity)
            .despawn();
    }
}