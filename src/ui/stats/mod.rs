use bevy::app::App;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::{AssetServer, BuildChildren, Commands, Component, NodeBundle, Plugin, Query, Res, Style, SystemSet, Text, TextBundle, With};
use bevy::text::{TextSection, TextStyle};
use bevy::ui::{AlignItems, Display, FlexDirection};
use bevy::utils::default;
use crate::player::Player;
use crate::r#match::state::MatchState;
use crate::state::GameState;

#[derive(Component)]
struct UITag;

#[derive(Component)]
struct UITagPoints;

#[derive(Component)]
struct UITagBlocks;

#[derive(Component)]
struct UITagBounces;

#[derive(Component)]
struct UITagWallHits;

#[derive(Component)]
struct UITagCombos;

#[derive(Component)]
struct UITagBalls;

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
                    .with_system(ui_update_points)
                    .with_system(ui_update_blocks)
                    .with_system(ui_update_bounces)
                    .with_system(ui_update_wall_hits)
                    .with_system(ui_update_combos)
                    .with_system(ui_update_balls)
            )

        ;
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
                ])).insert(UITagPoints);
            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Blocks: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UITagBlocks);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Bounces: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UITagBounces);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Wall Hits: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UITagWallHits);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Combos: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UITagCombos);

            parent
                .spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Balls: ", style.clone(),
                    ),
                    TextSection::from_style(style.clone())
                ])).insert(UITagBalls);
        })
        .insert(UITag);
    ;
}


fn ui_update_points(
    stats: Res<MatchState>,
    mut ui: Query<&mut Text, With<UITagPoints>>
) {
    match ui.get_single_mut() {
        Ok(mut text) => {
            text.sections[1].value = format!("{}", stats.points);
        }
        _ => {}
    }
}

fn ui_update_blocks(
    stats: Res<MatchState>,
    mut ui: Query<&mut Text, With<UITagBlocks>>
) {
    match ui.get_single_mut() {
        Ok(mut text) => {
            text.sections[1].value = format!("{}", stats.blocks);
        }
        _ => {}
    }
}

fn ui_update_bounces(
    stats: Res<MatchState>,
    mut ui: Query<&mut Text, With<UITagBounces>>
) {
    match ui.get_single_mut() {
        Ok(mut text) => {
            text.sections[1].value = format!("{}", stats.paddle_bounces);
        }
        _ => {}
    }
}

fn ui_update_wall_hits(
    stats: Res<MatchState>,
    mut ui: Query<&mut Text, With<UITagWallHits>>
) {
    match ui.get_single_mut() {
        Ok(mut text) => {
            text.sections[1].value = format!("{}", stats.wall_hits);
        }
        _ => {}
    }
}

fn ui_update_combos(
    stats: Res<MatchState>,
    mut ui: Query<&mut Text, With<UITagCombos>>
) {
    match ui.get_single_mut() {
        Ok(mut text) => {
            text.sections[1].value = format!("{}x, {}x", stats.paddle_bounce_combo, stats.single_bounce_combo);
        }
        _ => {}
    }
}

fn ui_update_balls(
    stats: Res<Player>,
    mut ui: Query<&mut Text, With<UITagBalls>>
) {
    match ui.get_single_mut() {
        Ok(mut text) => {
            text.sections[1].value = format!("{}", stats.balls_available);
        }
        _ => {}
    }
}
