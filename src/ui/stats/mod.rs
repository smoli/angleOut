use bevy::app::{App, Plugin, Update};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{default, in_state, AlignItems, ChildSpawnerCommands, Color, Commands, Component, Display, Entity, FlexDirection, IntoScheduleConfigs, Node, OnEnter, OnExit, Query, Res, Text, TextColor, TextFont, With};
use bevy::text::{FontSize, TextSpan};
use bevy_rapier3d::prelude::Velocity;
use crate::ball::Ball;
use crate::config::DEBUG_INFO_ENABLED;

use crate::player::Player;
use crate::r#match::state::MatchState;
use crate::state::GameState;

/// The match's readout, marked so it can go when the match does.
///
/// Named for what it is rather than `UITag`, because `c0013`'s playtest ends by
/// going back to the editor rather than through `PostMatch`, and the editor has
/// to be able to say what it is clearing away - see
/// [`playtest_teardown`](crate::editor::playtest::playtest_teardown).
#[derive(Component)]
pub struct MatchStatsUI;


#[derive(Component)]
enum UIInfoTag {
    MatchPoints,
    PlayerPoints,
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
    BallSpeedZ,
    FPS,
}


pub struct UIStatsPlugin;

impl Plugin for UIStatsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InMatch), ui_spawn)
            .add_systems(Update, ui_update_infos.run_if(in_state(GameState::InMatch)))
            .add_systems(OnExit(GameState::PostMatch), ui_despawn)
        ;
    }
}


fn ui_despawn(
    mut commands: Commands,
    ui: Query<Entity, With<MatchStatsUI>>,
) {
    for ui in &ui {
        //info!("Despawn stats ui {:?}", ui);
        commands.entity(ui)
            .despawn();
    }
}

fn ui_update_infos(
    match_stats: Res<MatchState>,
    player_stats: Query<&Player>,
    mut ui: Query<(&mut TextSpan, &UIInfoTag)>,
    balls: Query<&Velocity, With<Ball>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(player) = player_stats.single() else { return; };

    for (mut span, tag) in &mut ui {
        match tag {
            UIInfoTag::FPS => **span = {
                if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                    if let Some(fps) = fps.smoothed() {
                        format!("{:.1}", fps)
                    } else {
                        "no fps".to_string()
                    }
                } else {
                    "no fps".to_string()
                }
            },
            UIInfoTag::MatchPoints => **span = format!("{}", match_stats.points),
            UIInfoTag::Blocks => **span = format!("{}", match_stats.blocks),
            UIInfoTag::Bounces => **span = format!("{}", match_stats.paddle_bounces),
            UIInfoTag::WallHits => **span = format!("{}", match_stats.wall_hits),
            UIInfoTag::Combos => **span = format!("{}x, {}x", match_stats.paddle_bounce_combo, match_stats.single_bounce_combo),
            UIInfoTag::PlayerPoints => **span = format!("{}", player.points),
            UIInfoTag::Balls => **span = format!("{}", player.balls_available),
            UIInfoTag::BlocksHit => **span = format!("{}", match_stats.blocks_hit),
            UIInfoTag::BlocksLost => **span = format!("{}", match_stats.blocks_lost),
            UIInfoTag::BallsInPLay => **span = format!("{}", player.balls_in_play),
            UIInfoTag::BallsGrabbed => **span = format!("{}", player.balls_grabbed),
            UIInfoTag::BallsLost => **span = format!("{}", player.balls_lost),
            UIInfoTag::BallSpeed => {
                match balls.single() {
                    Ok(velo) => **span = format!("{}", velo.linear.length()),
                    Err(_) => **span = format!("No Ball")
                }
            }
            UIInfoTag::BallSpeedZ => {
                match balls.single() {
                    Ok(velo) => **span = format!("{}", velo.linear.z),
                    Err(_) => **span = format!("No Ball")
                }
            }
        }
    }
}


/// A stat line is a `Text` label with a `TextSpan` child holding the live value.
fn spawn_stat(parent: &mut ChildSpawnerCommands, label: &str, font: &TextFont, tag: UIInfoTag) {
    parent
        .spawn((
            Text::new(label.to_string()),
            font.clone(),
            TextColor(Color::WHITE),
        ))
        .with_child((TextSpan::default(), font.clone(), TextColor(Color::WHITE), tag));
}


fn ui_spawn(
    mut commands: Commands,
    asset_server: Res<bevy::asset::AssetServer>,
) {
    let font = TextFont {
        font: asset_server.load("fonts/Orbitron-Regular.ttf").into(),
        font_size: FontSize::Px(30.0),
        ..default()
    };

    commands
        .spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|parent| {
            spawn_stat(parent, "Player Points: ", &font, UIInfoTag::PlayerPoints);
            spawn_stat(parent, "Level Points: ", &font, UIInfoTag::MatchPoints);
            spawn_stat(parent, "Combos: ", &font, UIInfoTag::Combos);
            spawn_stat(parent, "Balls: ", &font, UIInfoTag::Balls);
            spawn_stat(parent, "Blocks hit: ", &font, UIInfoTag::BlocksHit);
            spawn_stat(parent, "Blocks lost: ", &font, UIInfoTag::BlocksLost);

            if DEBUG_INFO_ENABLED {
                spawn_stat(parent, "Balls Grabbed: ", &font, UIInfoTag::BallsGrabbed);
                spawn_stat(parent, "Blocks: ", &font, UIInfoTag::Blocks);
                spawn_stat(parent, "Bounces: ", &font, UIInfoTag::Bounces);
                spawn_stat(parent, "Wall Hits: ", &font, UIInfoTag::WallHits);
                spawn_stat(parent, "Balls in Play: ", &font, UIInfoTag::BallsInPLay);
                spawn_stat(parent, "Balls Lost: ", &font, UIInfoTag::BallsLost);
                spawn_stat(parent, "Ball Speed: ", &font, UIInfoTag::BallSpeed);
                spawn_stat(parent, "Ball Z: ", &font, UIInfoTag::BallSpeedZ);
            }

            spawn_stat(parent, "FPS: ", &font, UIInfoTag::FPS);
        })
        .insert(MatchStatsUI);
}
