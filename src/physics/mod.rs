use bevy::app::{App, CoreStage};
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{Commands, Component, Entity, EventReader, IntoSystemDescriptor, Plugin, Query, ResMut, Resource, SystemSet, SystemStage, Transform, With, Without};
use bevy::utils::HashMap;
use bevy_rapier3d::plugin::{NoUserData, PhysicsStages, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::{CollisionEvent, Sensor, Velocity};
#[allow(unused_imports)]
use bevy_rapier3d::render::RapierDebugRenderPlugin;

use crate::config::{BLOOM_ENABLED, DEBUG_PHYSICS_ENABLED};
use crate::labels::SystemLabels;
use crate::state::GameState;

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


pub const COLLISION_EVENT_HANDLING: &str = "STAGE_COLLISION_EVENT_HANDLING";

#[derive(Component)]
pub struct PhysicsPlugin;


impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(CollisionInfo {
                collisions: HashMap::new()
            })

            .add_system_to_stage(CoreStage::Update,
                                 handle_collision_events)

            .add_system_to_stage(CoreStage::Last,
                                 cleanup_collision_tags,
            )

            .add_stage_after(CoreStage::PostUpdate, COLLISION_EVENT_HANDLING, SystemStage::parallel())
            ;

        if DEBUG_PHYSICS_ENABLED && !BLOOM_ENABLED {
            app.add_plugin(RapierDebugRenderPlugin::default());
        }
        ;
    }
}

fn handle_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut collidables: Query<(&Collidable, &Transform)>,
    mut collisions: ResMut<CollisionInfo>,
    velocity: Query<&Velocity>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, flags) => {

                if let Ok((col_a, trans_a)) = collidables.get(*a) {
                    if let Ok((col_b, trans_b)) = collidables.get(*b) {

                        let vel_a = if let Ok(va) = velocity.get(*a) {
                            Some(va.linvel.clone())
                        } else { None };

                        let vel_b = if let Ok(vb) = velocity.get(*b) {
                            Some(vb.linvel.clone())
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


                        info!("Collision {:?}::{:?} - {:?}::{:?}", col_a.kind, a, col_b.kind, b);
                    }
                }
            }
            _ => {}
        }
    }
}

/*fn handle_contact_force_events(
    mut commands: Commands,
    mut contact_force_events: EventReader<ContactForceEvent>,
    collidables: Query<(&Collidable, &Transform), Without<Sensor>>,
) {
    for contact_force_event in contact_force_events.iter() {
        if let Ok((col_a, trans_a)) = collidables.get(contact_force_event.collider1) {
            if let Ok((col_b, trans_b)) = collidables.get(contact_force_event.collider2) {
                commands.entity(contact_force_event.collider1)
                    .remove::<CollisionTag>()
                    .insert(CollisionTag {
                        other: col_b.kind.clone(),
                        pos: trans_a.translation,
                        other_velocity: None,
                        other_pos: trans_b.translation,
                    });

                commands.entity(contact_force_event.collider2)
                    .remove::<CollisionTag>()
                    .insert(CollisionTag {
                        other: col_a.kind.clone(),
                        pos: trans_b.translation,
                        other_velocity: None,
                        other_pos: trans_a.translation,
                    });


                info!("Contact Force {:?}-{:?}", col_a.kind, col_b.kind);
            }
        }
    }
}
*/

fn cleanup_collision_tags(
    mut commands: Commands,
    mut collidables: Query<(Entity), With<CollisionTag>>,
    mut collisions: ResMut<CollisionInfo>,
) {
    for (collidable) in &mut collidables {
        commands.entity(collidable)
            .remove::<CollisionTag>();
    }
    if collisions.collisions.len() != 0 {
        info!("Clear");
        collisions.clear();
    }
}