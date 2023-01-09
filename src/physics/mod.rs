use bevy::app::App;
use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::prelude::{Plugin, Component, Commands, EventReader, Query, Entity, With, EventWriter, SystemSet, IntoSystemDescriptor};
use bevy_rapier3d::plugin::{RapierPhysicsPlugin, NoUserData};
use bevy_rapier3d::prelude::CollisionEvent;
use crate::labels::SystemLabels;
use crate::state::GameState;

#[derive(Clone)]
pub enum CollidableKind {
    Ball,
    Wall,
    Ship,
    Block
}

#[derive(Component)]
pub struct Collidable {
    pub kind: CollidableKind
}

#[derive(Component)]
pub struct CollisionTag {
    pub other: CollidableKind
}

#[derive(Component)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(handle_collision_events.before(SystemLabels::UpdateWorld))
                    .with_system(cleanup_collision_tags.after(SystemLabels::UpdateState))
            )
        // .add_plugin(RapierDebugRenderPlugin::default())
        ;
    }
}

fn handle_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    collidables: Query<&Collidable>
) {
    for collision_event in collision_events.iter() {
        // println!("Received collision event: {:?}", collision_event);
        // println!("{:?}", collision_event);

        let mut combo: Option<(CollidableKind, CollidableKind)> = None;

        match collision_event {
            CollisionEvent::Started(a, b, _) => {
                if let Ok(col_a) = collidables.get(*a) {
                    if let Ok(col_b) = collidables.get(*b) {
                        commands.entity(*a)
                            .remove::<CollisionTag>()
                            .insert(CollisionTag {
                                other: col_b.kind.clone()
                            });
                        commands.entity(*b)
                            .remove::<CollisionTag>()
                            .insert(CollisionTag {
                                other: col_a.kind.clone()
                            });

                        combo = Some((col_a.kind.clone(), col_b.kind.clone()));
                    }
                }
            }
            _=> {}
        }
    }
}


fn cleanup_collision_tags(
    mut commands: Commands,
    collidables: Query<Entity, With<CollisionTag>>
) {
    for collidable in &collidables {
        commands.entity(collidable)
            .remove::<CollisionTag>();
    }
}