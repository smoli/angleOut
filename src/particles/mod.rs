use bevy::app::{App, Plugin, PostUpdate};
use bevy::prelude::{Assets, Commands, Component, Entity, IntoScheduleConfigs, Name, OnEnter, OnExit, Query, Res, ResMut, Transform, Vec3, Vec4, With, Without};
use bevy_hanabi::prelude::*;

use crate::ball::Ball;
use crate::block::{Block, Hittable};
use crate::physics::{CollidableKind, CollisionEventHandling, CollisionInfo, CollisionTag};
use crate::state::GameState;

#[derive(Component)]
struct ImpactEffect;

#[derive(Component)]
struct TrailEffect;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(OnEnter(GameState::InMatch), particles_setup_block_impact)

            .add_systems(PostUpdate, particle_handle_block_ball.in_set(CollisionEventHandling))

            .add_systems(OnExit(GameState::InMatch), particles_despawn_all)
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

    // Emit only when `EffectSpawner::reset()` is called on a block hit.
    let spawner = SpawnerSettings::once(20.0.into()).with_emit_on_start(false);

    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(25.0).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.0).expr());

    let effect = effects.add(
        EffectAsset::new(32768, spawner, writer.finish())
            .with_name("BallBlockImpact")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::new(1.0, 0.5, 1.0)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier {
                gradient,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(OrientModifier {
                mode: OrientMode::FaceCameraPosition,
                rotation: None,
            })
    );

    commands
        .spawn(ParticleEffect::new(effect))
        .insert(Name::new("effect"))
        .insert(ImpactEffect);
}


fn particle_handle_block_ball(
    blocks: Query<Entity, (With<Block>, With<CollisionTag>, With<Hittable>)>,
    mut effect: Query<(&mut EffectSpawner, &mut Transform), (Without<Block>, With<ImpactEffect>)>,
    collisions: Res<CollisionInfo>,
) {
    // `EffectSpawner` is only added in PostUpdate the frame after the effect entity
    // is spawned, so it can legitimately be missing for a frame.
    let Ok((mut effect_spawner, mut effect_transform)) = effect.single_mut() else {
        return;
    };

    for block in &blocks {
        if let Some(collision) = collisions.collisions.get(&block) {
            for collision in collision {
                if collision.other == CollidableKind::Ball {
                    effect_transform.translation = collision.other_pos.clone();
                    effect_spawner.reset();
                }
            }
        }
    }
}


#[allow(dead_code)]
fn particles_setup_ball_trail(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.5, 0.5, 0.5, 1.0));
    gradient.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.0));

    let spawner = SpawnerSettings::rate(20.0.into());

    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(25.0).expr(),
    };

    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.6).expr());

    let effect = effects.add(
        EffectAsset::new(32768, spawner, writer.finish())
            .with_name("BallTrail")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec3::new(1.0, 0.5, 1.0)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier {
                gradient,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(OrientModifier {
                mode: OrientMode::FaceCameraPosition,
                rotation: None,
            })
    );

    commands
        .spawn(ParticleEffect::new(effect))
        .insert(Name::new("trail"))
        .insert(TrailEffect);
}


#[allow(dead_code)]
fn particle_handle_ball_trail(
    balls: Query<&Transform, With<Ball>>,
    mut effect: Query<&mut Transform, (Without<Ball>, With<TrailEffect>)>,
) {
    let Ok(mut effect_transform) = effect.single_mut() else { return; };

    for trans in &balls {
        effect_transform.translation = trans.translation.clone();
    }
}


fn particles_despawn_all(
    mut commands: Commands,
    effects: Query<Entity, With<ImpactEffect>>,
) {
    for effect in &effects {
        //info!("Despawn particle effect");
        commands.entity(effect)
            .despawn();
    }
}
