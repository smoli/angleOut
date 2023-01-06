use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::prelude::{Color, Commands, Component, Plugin, Query, Res, TextBundle, With};
use bevy::text::{Text, TextSection, TextStyle};
use crate::states::GameState;

#[derive(Component)]
pub struct UIStats;


pub struct UIStatsPlugin;

impl Plugin for UIStatsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_stats_ui)
            .add_system(update_stats_ui)
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
                    UIStats
    ));
}

fn update_stats_ui(gameState: Res<GameState>, mut texts: Query<&mut Text, With<UIStats>>) {
    for mut text in &mut texts {
        text.sections[1].value = format!("{}", gameState.blocks);
    }
}