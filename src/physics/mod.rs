use bevy::app::App;
use bevy::log::{info};
use bevy::math::Vec3;
use bevy::prelude::{Plugin, Component, Commands, EventReader, Query, Entity, With, SystemSet, IntoSystemDescriptor, Without, Transform};
use bevy_rapier3d::plugin::{RapierPhysicsPlugin, NoUserData};
use bevy_rapier3d::prelude::{CollisionEvent, ContactForceEvent, Sensor, Velocity};
use crate::labels::SystemLabels;
use crate::state::GameState;

#[allow(unused_imports)]
use bevy_rapier3d::render::RapierDebugRenderPlugin;


#[derive(Clone, Debug, PartialEq)]
pub enum CollidableKind {
    Ball,
    Wall,
    DeathTrigger,
    Ship,
    Block,
    Pickup
}

#[derive(Component)]
pub struct Collidable {
    pub kind: CollidableKind,
}

#[derive(Component)]
pub struct CollisionTag {
    pub other: CollidableKind,
    pub pos: Vec3,
    pub other_velocity: Option<Vec3>
}


#[derive(Component)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(handle_collision_events.before(SystemLabels::UpdateWorld))
                    .with_system(handle_contact_force_events.before(SystemLabels::UpdateWorld))
                    .with_system(cleanup_collision_tags.after(SystemLabels::UpdateState))
            )
        .add_plugin(RapierDebugRenderPlugin::default())
        ;
    }
}

fn handle_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    collidables: Query<(&Collidable, &Transform), Without<Sensor>>,
    velocity: Query<&Velocity>
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(a, b, _) => {

                if let Ok((col_a, trans_a)) = collidables.get(*a) {
                    if let Ok((col_b, trans_b)) = collidables.get(*b) {

                        let vel_a = if let Ok(va) = velocity.get(*a) {
                            Some(va.linvel.clone())
                        } else { None };

                        let vel_b = if let Ok(vb) = velocity.get(*b) {
                            Some(vb.linvel.clone())
                        } else { None };


                        commands.entity(*a)
                            .remove::<CollisionTag>()
                            .insert(CollisionTag {
                                other: col_b.kind.clone(),
                                pos: trans_a.translation,
                                other_velocity: vel_b
                            });

                        commands.entity(*b)
                            .remove::<CollisionTag>()
                            .insert(CollisionTag {
                                other: col_a.kind.clone(),
                                pos: trans_b.translation,
                                other_velocity: vel_a
                            });


                        if col_a.kind != CollidableKind::Block {
                            info!("Collision {:?}-{:?}", col_a.kind, col_b.kind);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_contact_force_events(
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
                        other_velocity: None
                    });

                commands.entity(contact_force_event.collider2)
                    .remove::<CollisionTag>()
                    .insert(CollisionTag {
                        other: col_a.kind.clone(),
                        pos: trans_b.translation,
                        other_velocity: None
                    });


                info!("Contact Force {:?}-{:?}", col_a.kind, col_b.kind);
            }
        }
    }
}


fn cleanup_collision_tags(
    mut commands: Commands,
    collidables: Query<Entity, With<CollisionTag>>,
) {
    for collidable in &collidables {
        commands.entity(collidable)
            .remove::<CollisionTag>();
    }
}