use bevy::app::App;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::log::info;
use bevy::prelude::{AssetServer, BuildChildren, Commands, Component, Entity, NodeBundle, Plugin, Query, Res, Style, SystemSet, Text, TextBundle, With, Without};
use bevy::text::{TextSection, TextStyle};
use bevy::ui::{AlignItems, Display, FlexDirection};
use bevy::utils::default;
use bevy_rapier3d::prelude::Velocity;
use crate::ball::Ball;
use crate::config::DEBUG_INFO_ENABLED;

use crate::player::Player;
use crate::r#match::state::MatchState;
use crate::state::GameState;

#[derive(Component)]
struct UITag;


#[derive(Component)]
enum UIInfoTag {
    Points,
    Blocks,
    Bounces,
    WallHits,
    Combos,
    Balls,
    BlocksHit,
    BlocksLost,
    BallsInPLay,
    BallsGrabbed,
    BallsLost,
    BallSpeed,
    BallSpeedZ
}


pub struct UIStatsPlugin;

impl Plugin for UIStatsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(ui_spawn)
            )
            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(ui_update_infos)
            )
            .add_system_set(
                SystemSet::on_exit(GameState::InMatch)
                    .with_system(ui_despawn)
            )
        ;
    }
}



fn ui_despawn(
    mut commands: Commands,
    ui: Query<Entity, With<UIInfoTag>>
) {
    for ui in &ui {
        info!("Despawn stats ui {:?}", ui);
        commands.entity(ui)
            .despawn_recursive();
    }
}

fn ui_update_infos(
    match_stats: Res<MatchState>,
    player_stats: Query<&Player>,
    mut ui: Query<(&mut Text, &UIInfoTag), Without<Ball>>,
    balls: Query<&Velocity, With<Ball>>
) {
    let player = player_stats.get_single().unwrap();

    for (mut text, tag) in &mut ui {
        match tag {
            UIInfoTag::Points => text.sections[1].value = format!("{}", match_stats.points),
            UIInfoTag::Blocks => text.sections[1].value = format!("{}", match_stats.blocks),
            UIInfoTag::Bounces => text.sections[1].value = format!("{}", match_stats.paddle_bounces),
            UIInfoTag::WallHits => text.sections[1].value = format!("{}", match_stats.wall_hits),
            UIInfoTag::Combos => text.sections[1].value = format!("{}x, {}x", match_stats.paddle_bounce_combo, match_stats.single_bounce_combo),
            UIInfoTag::Balls => text.sections[1].value = format!("{}", player.balls_available),
            UIInfoTag::BlocksHit => text.sections[1].value = format!("{}", match_stats.blocks_hit),
            UIInfoTag::BlocksLost => text.sections[1].value = format!("{}", match_stats.blocks_lost),
            UIInfoTag::BallsInPLay => text.sections[1].value = format!("{}", player.balls_in_play),
            UIInfoTag::BallsGrabbed => text.sections[1].value = format!("{}", player.balls_grabbed),
            UIInfoTag::BallsLost => text.sections[1].value = format!("{}", player.balls_lost),
            UIInfoTag::BallSpeed => {
                match balls.get_single() {
                    Ok(velo) => text.sections[1].value = format!("{}", velo.linvel.length()),
                    Err(_) => text.sections[1].value = format!("No Ball")
                }
            }
            UIInfoTag::BallSpeedZ => {
                match balls.get_single() {
                    Ok(velo) => text.sections[1].value = format!("{}", velo.linvel.z),
                    Err(_) => text.sections[1].value = format!("No Ball")
                }
            }

        }
    }
}



fn ui_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let style = TextStyle {
        font: asset_server.load("fonts/Orbitron-Regular.ttf"),
        font_size: 30.0,
        color: Default::default(),
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Points: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::Points);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Combos: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::Combos);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Balls: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::Balls);

            if DEBUG_INFO_ENABLED {
            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Blocks: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::Blocks);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Bounces: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::Bounces);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Wall Hits: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::WallHits);




            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Balls in Play: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::BallsInPLay);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Balls Lost: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::BallsLost);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Balls Grabbed: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::BallsGrabbed);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Blocks hit: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::BlocksHit);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Blocks lost: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::BlocksLost);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Ball Speed: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::BallSpeed);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Ball Z: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UIInfoTag::BallSpeedZ);
            }
        })
        .insert(UITag);

}



