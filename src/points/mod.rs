use std::f32::consts::{PI, TAU};
use std::time::Duration;
use bevy::app::{App, Plugin};
use bevy::asset::{AssetServer, LoadState};
use bevy::log::{info, warn};
use bevy::math::Vec3;
use bevy::pbr::{NotShadowCaster, PbrBundle};
use bevy::prelude::{AlphaMode, Assets, BuildChildren, Commands, Component, ComputedVisibility, default, DespawnRecursiveExt, Entity, Handle, Image, Mesh, Quat, Query, Res, ResMut, Resource, shape, SpatialBundle, SpriteSheetBundle, StandardMaterial, SystemSet, TextureAtlas, TextureAtlasSprite, TimerMode, Transform, TransformBundle, Vec2, Visibility, With};
use bevy::time::{Time, Timer};
use bevy_rapier3d::rapier::crossbeam::channel::at;
use bevy_sprite3d::{AtlasSprite3d, AtlasSprite3dBundle, Sprite3d, Sprite3dParams, Sprite3dPlugin};
use crate::state::GameState;

pub struct PointsPlugin;

#[derive(Component)]
pub struct PointsDisplayRequest;


#[derive(Resource)]
struct PointsResources {
    atlas: Handle<TextureAtlas>,
}

#[derive(Component)]
pub struct PointsDisplay {
    pub text: String,
    pub position: Vec3,
}

#[derive(Component)]
struct FuseTimer {
    timer: Timer,
}

impl Plugin for PointsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(Sprite3dPlugin)
            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(point_setup)
            )

            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(points_handle_requests)
                    .with_system(points_update)
            )

        /*            .add_system_set(
                        SystemSet::on_exit(GameState::InMatch)
                            .with_system(points_remove_all)
                    )*/

        ;
    }
}

fn point_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = asset_server.load("Points.png");

    let texture_atlas =
        TextureAtlas::from_grid(image, Vec2::new(128.0, 128.0), 10, 2, None, None);

    let mut r = PointsResources {
        atlas: texture_atlases.add(texture_atlas)
    };

    /*   // Test
   commands.spawn(PointsDisplay {
       text: "12345".to_string(),
       position: Default::default(),
   }).insert(PointsDisplayRequest)
   ;*/

    commands.insert_resource(r);
}


fn get_sprite_indexes(text: &String) -> Vec<usize> {
    let mut r = vec![];
    for c in text.chars() {
        match c {
            '0' => r.push(9),
            '1' => r.push(0),
            '2' => r.push(1),
            '3' => r.push(2),
            '4' => r.push(3),
            '5' => r.push(4),
            '6' => r.push(5),
            '7' => r.push(6),
            '8' => r.push(7),
            '9' => r.push(8),
            _ => {}
        }
    }

    r
}


fn points_update(
    mut commands: Commands,
    time: Res<Time>,
    mut points: Query<(Entity, &mut FuseTimer, &mut Transform)>,
) {
    for (points, mut fuse, mut trans) in &mut points {
        fuse.timer.tick(time.delta());

        if fuse.timer.finished() {
            commands.entity(points)
                .despawn_recursive();
        } else {
            trans.translation = trans.translation + Vec3::new(0.0, -0.35, 0.0);
        }
    }
}

fn points_handle_requests(
    mut commands: Commands,
    points_resource: Res<PointsResources>,
    requests: Query<(Entity, &PointsDisplay), With<PointsDisplayRequest>>,
    mut sprite_params: Sprite3dParams,
    asset_server: Res<AssetServer>,
) {
    /*
        if asset_server.get_load_state(&points_resource.atlas) != LoadState::Loaded {
            return;
        }*/

    for (entity, points) in &requests {
        info!("Points request");

        let idx = get_sprite_indexes(&points.text);

        let char_size = 3.5;

        commands
            .entity(entity)
            .remove::<PointsDisplayRequest>()
            .with_children(|parent| {
                let mut x: f32 = -1.0 * idx.len() as f32 * char_size / 2.0;
                let mut z: f32 = -0.1;
                for i in idx {
                    parent.spawn(AtlasSprite3d {
                        transform: Transform::from_xyz(x, 0.0, z),
                        atlas: points_resource.atlas.clone(),
                        index: i,
                        pixels_per_metre: 10.0,
                        pivot: None,
                        partial_alpha: true,
                        unlit: true,
                        double_sided: false,
                        emissive: Default::default(),
                    }.bundle(&mut sprite_params))
                        .insert(NotShadowCaster);

                    x += char_size;
                    z += -0.1;
                }
            })
            .insert(SpatialBundle::from_transform(
                Transform::from_rotation(Quat::from_rotation_x(-PI * 0.5)).with_translation(points.position.clone())
            ))
            .insert(FuseTimer {
                timer: Timer::new(Duration::from_secs(2), TimerMode::Once)
            })
        ;
    }
}