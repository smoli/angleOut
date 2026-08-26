use bevy::app::{App, Last, PostUpdate, Update};
use bevy::math::Vec3;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Component, Entity, IntoScheduleConfigs, MessageReader, Plugin, Query, ResMut, Resource, SystemSet, Transform, With};
use bevy_rapier3d::plugin::{NoUserData, PhysicsSet, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::{CollisionEvent, Velocity};
#[allow(unused_imports)]
use bevy_rapier3d::render::RapierDebugRenderPlugin;

use crate::config::{BLOOM_ENABLED, DEBUG_PHYSICS_ENABLED};

#[derive(Clone, Debug, PartialEq)]
pub enum CollidableKind {
    Ball,
    Wall,
    DeathTrigger,
    DirectionalDeathTrigger(Vec3),
    Ship,
    Block,
    Pickup,
}

#[derive(Component)]
pub struct Collidable {
    pub kind: CollidableKind,
}

#[derive(Component)]
pub struct CollisionTag;

#[derive(Debug, Clone, PartialEq)]
pub struct Collision {
    pub other_entity: Entity,
    pub other: CollidableKind,
    pub pos: Vec3,
    pub other_velocity: Option<Vec3>,
    pub other_pos: Vec3,
}

#[derive(Resource)]
pub struct CollisionInfo {
    pub collisions: HashMap<Entity, Vec<Collision>>,
}

impl CollisionInfo {
    pub fn clear(&mut self) {
        self.collisions.clear();
    }

    pub fn insert(&mut self, entity: Entity, other_entity: Entity, other: CollidableKind, pos: Vec3, other_velocity: Option<Vec3>, other_pos: Vec3) {
        let info = Collision {
            other_entity,
            other,
            pos,
            other_velocity,
            other_pos,
        };


        self.collisions
            .entry(entity)
            .or_insert(Vec::new())
            .push(info);
    }
}


/// Runs in `PostUpdate` once rapier has written back this frame's simulation
/// results, which is where all the per-domain collision reactions live.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionEventHandling;

pub struct PhysicsPlugin;


impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(CollisionInfo {
                collisions: HashMap::new()
            })

            .configure_sets(
                PostUpdate,
                CollisionEventHandling.after(PhysicsSet::Writeback),
            )

            .add_systems(Update, handle_collision_events)

            .add_systems(Last, cleanup_collision_tags)
            ;

        if DEBUG_PHYSICS_ENABLED && !BLOOM_ENABLED {
            app.add_plugins(RapierDebugRenderPlugin::default());
        }
        ;
    }
}

fn handle_collision_events(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    collidables: Query<(&Collidable, &Transform)>,
    mut collisions: ResMut<CollisionInfo>,
    velocity: Query<&Velocity>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(a, b, _flags) => {

                if let Ok((col_a, trans_a)) = collidables.get(*a) {
                    if let Ok((col_b, trans_b)) = collidables.get(*b) {

                        let vel_a = if let Ok(va) = velocity.get(*a) {
                            Some(va.linear.clone())
                        } else { None };

                        let vel_b = if let Ok(vb) = velocity.get(*b) {
                            Some(vb.linear.clone())
                        } else { None };


                        commands.entity(*a)
                            .insert(CollisionTag);

                        collisions.insert(
                            *a,
                            *b,
                            col_b.kind.clone(),
                            trans_a.translation,
                            vel_b,
                            trans_b.translation);


                        commands.entity(*b)
                            .insert(CollisionTag);

                        collisions.insert(
                            *b,
                            *a,
                            col_a.kind.clone(),
                            trans_b.translation,
                            vel_a,
                            trans_a.translation);


                        //info!("Collision {:?}::{:?} - {:?}::{:?}", col_a.kind, a, col_b.kind, b);
                    }
                }
            }
            _ => {}
        }
    }
}

fn cleanup_collision_tags(
    mut commands: Commands,
    collidables: Query<Entity, With<CollisionTag>>,
    mut collisions: ResMut<CollisionInfo>,
) {
    for collidable in &collidables {
        commands.entity(collidable)
            .remove::<CollisionTag>();
    }
    if collisions.collisions.len() != 0 {
        //info!("Clear");
        collisions.clear();
    }
}
