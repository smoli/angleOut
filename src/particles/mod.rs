use bevy::app::App;
use bevy::core::Name;
use bevy::log::info;
use bevy::prelude::{Commands, Assets, DespawnRecursiveExt, Entity, Plugin, ResMut, SystemSet, Transform, Vec2, Vec4, With, Query, Without};

use bevy_hanabi::{BillboardModifier, ColorOverLifetimeModifier, EffectAsset, Gradient, HanabiPlugin, ParticleEffect, ParticleEffectBundle, PositionCircleModifier, PositionSphereModifier, ShapeDimension, SizeOverLifetimeModifier, Spawner};
use crate::block::Block;
use crate::config::BLOCK_DEPTH;
use crate::physics::{CollidableKind, CollisionTag};
use crate::state::GameState;


use bevy::{
    prelude::*,
    render::settings::{WgpuFeatures, WgpuSettings},
    sprite::Anchor,
};
use crate::ball::Ball;


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

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(particle_handle_block_ball)
            )

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
    gradient.add_key(0.0, Vec4::new(0.5, 0.5, 0.5, 1.0));
    gradient.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.0));

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
        .insert(Name::new("effect"));
}


fn particle_handle_block_ball(
    blocks: Query<(&CollisionTag, &Transform), With<Ball>>,
    mut effect: Query<(&mut ParticleEffect, &mut Transform), Without<Ball>>,
) {
    let (mut effect, mut effect_transform) = effect.single_mut();

    for (collision, trans) in &blocks {
        if collision.other == CollidableKind::Block {
            effect_transform.translation = trans.translation.clone();
            effect.maybe_spawner().unwrap().reset();
        }

    }
}


fn particles_despawn_all(
    mut commands: Commands,
    effects: Query<Entity, With<ParticleEffect>>
) {
    /*for effect in &effects {
        commands.entity(effect)
            .despawn_recursive();
    }*/
}