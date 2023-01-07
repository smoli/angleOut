use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::prelude::{BuildChildren, Color, Commands, Component, Plugin, Query, Res, TextBundle, Transform, UiRect, Vec3, With};
use bevy::prelude::Keyframes::Translation;
use bevy::text::{Text, TextSection, TextStyle};
use bevy::ui::{PositionType, Style, Val};
use bevy::utils::default;
use crate::states::MatchState;

#[derive(Component)]
pub struct UIStatsBlocks;

#[derive(Component)]
pub struct UIStatsBounces;


pub struct UIStatsPlugin;

impl Plugin for UIStatsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_stats_ui)
            .add_system(update_stats_ui_blocks)
            .add_system(update_stats_ui_bounces)
        ;
    }
}


fn setup_stats_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((TextBundle::from_sections([
        TextSection::new(
            "Blocks: ",
            TextStyle {
                font: asset_server.load("BAUHS93.TTF"),
                font_size: 60.0,
                color: Color::GOLD,
            },
        ), TextSection::from_style(
            TextStyle {
                font: asset_server.load("BAUHS93.TTF"),
                font_size: 60.0,
                color: Color::GOLD,
            }
        )
    ]),
                    UIStatsBlocks
    ));

    commands.spawn((TextBundle {
        node: Default::default(),
        style: Style::from(Style {

            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(0.0),
                top: Val::Px(80.0),
                ..default()
            },
            ..default()
        }),
        text: Text::from_sections([TextSection::new(
            "Bounces: ",
            TextStyle {
                font: asset_server.load("BAUHS93.TTF"),
                font_size: 60.0,
                color: Color::GOLD,
            },
        ), TextSection::from_style(
            TextStyle {
                font: asset_server.load("BAUHS93.TTF"),
                font_size: 60.0,
                color: Color::GOLD,
            }
        )]),
        ..default()
    },
            UIStatsBounces
    ));

}

fn update_stats_ui_blocks(match_state: Res<MatchState>, mut texts: Query<&mut Text, With<UIStatsBlocks>>) {
    for mut text in &mut texts {
        text.sections[1].value = format!("{}", match_state.blocks);
    }
}

fn update_stats_ui_bounces(match_state: Res<MatchState>, mut texts: Query<&mut Text, With<UIStatsBounces>>) {
    for mut text in &mut texts {
        text.sections[1].value = format!("{}", match_state.paddle_bounces);
    }
}