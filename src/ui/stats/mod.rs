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
enum UIInfoTag {
    Points,
    Blocks,
    Bounces,
    WallHits,
    Combos,
    Balls,
    BlocksHit,
    BlocksLost
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
                ])).insert(UIInfoTag::Points);
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

        })
        .insert(UITag);
    ;
}



fn ui_update_infos(
    stats: Res<MatchState>,
    mut ui: Query<(&mut Text, &UIInfoTag)>

) {
    for (mut text, tag) in &mut ui {
        match tag {
            UIInfoTag::Points => text.sections[1].value = format!("{}", stats.points),
            UIInfoTag::Blocks => text.sections[1].value = format!("{}", stats.blocks),
            UIInfoTag::Bounces => text.sections[1].value = format!("{}", stats.paddle_bounces),
            UIInfoTag::WallHits => text.sections[1].value = format!("{}", stats.wall_hits),
            UIInfoTag::Combos => text.sections[1].value = format!("{}x, {}x", stats.paddle_bounce_combo, stats.single_bounce_combo),
            UIInfoTag::Balls => text.sections[1].value = format!("{}", stats.balls),
            UIInfoTag::BlocksHit => text.sections[1].value = format!("{}", stats.blocks_hit),
            UIInfoTag::BlocksLost => text.sections[1].value = format!("{}", stats.blocks_lost),
        }
    }
}
