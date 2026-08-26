use std::f32::consts::PI;
use std::time::Duration;

use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetServer;
use bevy::image::{TextureAtlas, TextureAtlasLayout};
use bevy::light::NotShadowCaster;
use bevy::math::{UVec2, Vec3};
use bevy::prelude::{default, in_state, AlphaMode, Assets, ChildSpawnerCommands, Commands, Component, Entity, Handle, Image, IntoScheduleConfigs, OnEnter, Quat, Query, Res, ResMut, Resource, Sprite, Transform, Visibility, With};
use bevy::time::{Time, Timer, TimerMode};
use bevy_sprite3d::{Sprite3d, Sprite3dPlugin};

use crate::state::GameState;

pub struct PointsPlugin;

#[derive(Component)]
pub struct PointsDisplayRequest;


#[derive(Resource)]
struct PointsResources {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
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
            .add_plugins(Sprite3dPlugin)
            .add_systems(OnEnter(GameState::InMatch), point_setup)

            .add_systems(
                Update,
                (points_handle_requests, points_update)
                    .run_if(in_state(GameState::InMatch)),
            )

        /*            .add_systems(OnExit(GameState::InMatch), points_remove_all) */

        ;
    }
}

fn point_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = asset_server.load("Points.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::new(128, 128), 10, 2, None, None);

    let r = PointsResources {
        image,
        layout: texture_atlases.add(layout),
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

        if fuse.timer.is_finished() {
            //info!("Despawn points display {:?}", points);
            commands.entity(points)
                .despawn();
        } else {
            trans.translation = trans.translation + Vec3::new(0.0, -0.35, 0.0);
        }
    }
}

fn points_handle_requests(
    mut commands: Commands,
    points_resource: Res<PointsResources>,
    requests: Query<(Entity, &PointsDisplay), With<PointsDisplayRequest>>,
) {
    for (entity, points) in &requests {
        //info!("Points request");

        let idx = get_sprite_indexes(&points.text);

        let char_size = 3.5;

        commands
            .entity(entity)
            .remove::<PointsDisplayRequest>()
            .insert(
                Transform::from_rotation(Quat::from_rotation_x(-PI * 0.5))
                    .with_translation(points.position.clone()),
            )
            .insert(Visibility::default())
            .insert(FuseTimer {
                timer: Timer::new(Duration::from_secs(2), TimerMode::Once)
            })
            .with_children(|parent: &mut ChildSpawnerCommands| {
                let mut x: f32 = -1.0 * idx.len() as f32 * char_size / 2.0;
                let z: f32 = -0.1;
                let mut count = 0.0;
                for i in idx {
                    parent.spawn((
                        Sprite {
                            image: points_resource.image.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: points_resource.layout.clone(),
                                index: i,
                            }),
                            ..default()
                        },
                        Sprite3d {
                            pixels_per_metre: 10.0,
                            pivot: None,
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            double_sided: false,
                            ..default()
                        },
                        Transform::from_xyz(x, 0.0, z * count),
                        NotShadowCaster,
                    ));

                    x += char_size;
                    count += 1.0;
                }
            })
        ;
    }
}
