use bevy::app::{App, Plugin};
use bevy::gltf::Gltf;
use bevy::prelude::{Assets, Commands, Component, DespawnRecursiveExt, Entity, EventReader, EventWriter, IntoSystemDescriptor, Query, Res, ResMut, SystemSet, Time, Transform, TransformBundle, Vec3, With};
use bevy::scene::SceneBundle;
use bevy::utils::default;
use bevy_rapier3d::dynamics::GravityScale;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollisionGroups, RigidBody};

use crate::config::{COLLIDER_GROUP_DEATH, COLLIDER_GROUP_PADDLE, COLLIDER_GROUP_PICKUP, PICKUP_GENERIC_SCENE, PICKUP_SPEED};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::level::{LevelDefinition, RequestTag};
use crate::MyAssetPack;
use crate::physics::{Collidable, CollidableKind, CollisionTag};
use crate::r#match::state::MatchState;
use crate::state::GameState;

pub struct PickupsPlugin;

impl Plugin for PickupsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(pickup_spawn.label(SystemLabels::UpdateWorld))
                    .with_system(pickup_update.label(SystemLabels::UpdateWorld))
                    .with_system(pickup_handle_collisions.label(SystemLabels::UpdateWorld))
                    .with_system(pickup_spawn_globals_on_event.label(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatch)
                    .with_system(pickup_despawn_all)
            )
        ;
    }
}


#[derive(Debug, Clone, Copy)]
pub enum PickupType {
    MoreBalls(i32),
    Grabber(i16)
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
    mut events: EventReader<MatchEvent>,
    mut match_state: ResMut<MatchState>,
    mut level: ResMut<LevelDefinition>,
) {
  //  let (player_entity, mut player, mut bouncer) = players.get_single_mut().unwrap();

    for ev in events.iter() {
        match ev {

            MatchEvent::TargetHit(p, block_type, behaviour) => {

                if let Some(pickup_type) = level.pickup_at(match_state.blocks as usize) {
                    commands.spawn(Pickup {
                        spawn_position: p.clone(),
                        pickup_type: pickup_type.clone()
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
        commands.entity(p)
            .despawn_recursive();
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


                .insert(SceneBundle {
                    scene: gltf.named_scenes[PICKUP_GENERIC_SCENE].clone(),
                    ..default()
                })
                .insert(TransformBundle::from_transform(Transform::from_translation(pickup.spawn_position.clone())))
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
        trans.translation += fall.dir * time.delta_seconds();
    }
}

fn pickup_handle_collisions(
    mut commands: Commands,
    pickups: Query<(Entity, &Pickup, &CollisionTag)>,
    mut events: EventWriter<MatchEvent>
) {
    for (entity, pickup, collision) in &pickups {
        match collision.other {
            CollidableKind::Ship => {
                events.send(MatchEvent::PickedUp(pickup.pickup_type))
            },

            CollidableKind::DeathTrigger => {

            },

            _ => {}
        }

        commands.entity(entity)
            .despawn_recursive();
    }
}