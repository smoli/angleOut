use bevy::app::App;
use bevy::ecs::query::QuerySingleError;
use bevy::prelude::{AssetServer, BuildChildren, Commands, Component, NodeBundle, Plugin, Query, Res, Style, SystemSet, Text, TextBundle, With};
use bevy::text::{TextSection, TextStyle};
use bevy::ui::{Display, FlexDirection};
use bevy::utils::default;
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

pub struct UIStatsPlugin;

impl Plugin for UIStatsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(ui_spawn)
            )
            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(ui_update_points)
                    .with_system(ui_update_blocks)
                    .with_system(ui_update_bounces)
                    .with_system(ui_update_wall_hits)
            )

        ;
    }
}


fn ui_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let style = TextStyle {
        font: asset_server.load("BAUHS93.TTF"),
        font_size: 20.0,
        color: Default::default(),
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
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
