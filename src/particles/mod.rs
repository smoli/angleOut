use bevy::{
    prelude::*,
    render::settings::{WgpuFeatures, WgpuSettings},
};
use bevy::app::App;
use bevy::core::Name;
use bevy::prelude::{Assets, Commands, Entity, Plugin, Query, ResMut, SystemSet, Transform, Vec2, Vec4, With, Without};
use bevy_hanabi::{BillboardModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, HanabiPlugin, ParticleEffect, ParticleEffectBundle, PositionSphereModifier, ShapeDimension, SizeOverLifetimeModifier, Spawner, Value};

use crate::ball::Ball;
use crate::block::{Block, Hittable};
use crate::physics::{CollidableKind, Collision, COLLISION_EVENT_HANDLING, CollisionInfo, CollisionTag};
use crate::state::GameState;

#[derive(Component)]
struct ImpactEffect;

#[derive(Component)]
struct TrailEffect;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        let mut options = WgpuSettings::default();
        options
            .features
            .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);


        app.add_plugin(HanabiPlugin)
            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(particles_setup_block_impact)
            )

            .add_system_to_stage(COLLISION_EVENT_HANDLING, particle_handle_block_ball)

            .add_system_set(
                SystemSet::on_exit(GameState::InMatch)
                    .with_system(particles_despawn_all)
            )
        ;
    }
}


fn particles_setup_block_impact(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0));
    gradient.add_key(0.5, Vec4::new(1.0, 1.0, 0.0, 1.0));
    gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

    let spawner = Spawner::once(20.0.into(), false);
    let effect = effects.add(
        EffectAsset {
            name: "BallBlockImpact".into(),
            capacity: 32768,
            spawner,
            ..Default::default()
        }
            .init(PositionSphereModifier {
                radius: 0.5,
                speed: 25.0.into(),
                dimension: ShapeDimension::Volume,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec2::new(1.0, 0.5)),
            })
            .render(ColorOverLifetimeModifier { gradient })
            .render(BillboardModifier {})
    );

    commands
        .spawn(ParticleEffectBundle::new(effect).with_spawner(spawner))
        .insert(Name::new("effect"))
        .insert(ImpactEffect);
}


fn particle_handle_block_ball(
    blocks: Query<(Entity), (With<Block>, With<CollisionTag>, With<Hittable>)>,
    mut effect: Query<(&mut ParticleEffect, &mut Transform), (Without<Block>, With<ImpactEffect>)>,
    collisions: Res<CollisionInfo>,
) {
    if effect.is_empty() {
        return
    }
    let (mut effect, mut effect_transform) = effect.single_mut();

    for block in &blocks {
        if let Some(collision) = collisions.collisions.get(&block) {
            for collision in collision {
                if collision.other == CollidableKind::Ball {
                    effect_transform.translation = collision.other_pos.clone();
                    effect.maybe_spawner().unwrap().reset();
                }
            }
        }
    }
}


fn particles_setup_ball_trail(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.5, 0.5, 0.5, 1.0));
    gradient.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.0));

    let spawner = Spawner::new(20.0.into(), Value::Single(1.0), Value::Single(0.6));
    let effect = effects.add(
        EffectAsset {
            name: "BallTrail".into(),
            capacity: 32768,
            spawner,
            ..Default::default()
        }
            .init(PositionSphereModifier {
                radius: 0.5,
                speed: 25.0.into(),
                dimension: ShapeDimension::Volume,
                ..Default::default()
            })
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec2::new(1.0, 0.5)),
            })
            .render(ColorOverLifetimeModifier { gradient })
            .render(BillboardModifier {})
    );

    commands
        .spawn(ParticleEffectBundle::new(effect).with_spawner(spawner))
        .insert(Name::new("trail"))
        .insert(TrailEffect);
}


fn particle_handle_ball_trail(
    balls: Query<(&Transform), With<Ball>>,
    mut effect: Query<(&mut ParticleEffect, &mut Transform), (Without<Ball>, With<TrailEffect>)>,
) {
    let (_, mut effect_transform) = effect.single_mut();

    for (trans) in &balls {
        effect_transform.translation = trans.translation.clone();
    }
}


fn particles_despawn_all(
    mut commands: Commands,
    effects: Query<Entity, With<ImpactEffect>>,
) {
    for effect in &effects {
        info!("Despawn particle effect");
        commands.entity(effect)
            .despawn_recursive();
    }
}