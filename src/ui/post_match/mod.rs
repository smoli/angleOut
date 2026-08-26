use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetServer;
use bevy::color::palettes::css::GOLD;
use bevy::prelude::{default, in_state, AlignSelf, Color, Commands, Component, Entity, FlexDirection, FlexWrap, GamepadButton, IntoScheduleConfigs, JustifyContent, MessageWriter, Node, OnEnter, OnExit, Query, Res, Text, TextColor, TextFont, Val, With};
use bevy::text::FontSize;
use bevy::ui::{BackgroundColor, ZIndex};
use leafwing_input_manager::prelude::{ActionState, InputMap};
use crate::actions::GameFlowActions;
use crate::events::GameFlowEvent;
use crate::player::{Player, PlayerState};
use crate::state::GameState;

pub struct PostMatchUIPlugin;


#[derive(Component)]
struct UITag;

impl Plugin for PostMatchUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::PostMatch), ui_spawn)

            .add_systems(Update, ui_handle_action.run_if(in_state(GameState::PostMatch)))

            .add_systems(OnExit(GameState::PostMatch), ui_despawn)
        ;
    }
}

fn ui_despawn(
    mut commands: Commands,
    ui: Query<Entity, With<UITag>>
) {
    for ui in &ui {
        //info!("Despawn post match ui {:?}", ui);
        commands.entity(ui)
            .despawn();
    }
}

fn ui_handle_action(
    actions: Query<&ActionState<GameFlowActions>, With<UITag>>,
    players: Query<&Player>,
    mut game_event: MessageWriter<GameFlowEvent>,
) {
    let Ok(player) = players.single() else { return; };

    for action in &actions {
        if action.just_released(&GameFlowActions::StartMatch) {
            //info!("Player requested Start!");
            match player.state {
                PlayerState::Open => {}
                PlayerState::HasWon => { game_event.write(GameFlowEvent::NextLevel); }
                PlayerState::HasLost => { game_event.write(GameFlowEvent::StartGame); }
            }
        }
    }
}

fn ui_spawn(
    players: Query<&Player>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let Ok(player) = players.single() else { return; };

    let headline = match player.state {
        PlayerState::Open => "You shouldn't be here!",
        PlayerState::HasWon => "You won!",
        PlayerState::HasLost => "You loose!"
    };

    let hint = match player.state {
        PlayerState::Open => "You shouldn't be here!",
        PlayerState::HasWon => "Press A/X to got to next level!",
        PlayerState::HasLost => "Press A/X to got to start again!"
    };

    commands
        .spawn((
            Node {
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                flex_wrap: FlexWrap::Wrap,
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
            ZIndex(100),
        ))

        .insert(UITag)
        .insert(
            InputMap::default()
                .with(GameFlowActions::StartMatch, GamepadButton::South),
        )

        .with_children(|parent| {
            parent.spawn((
                Text::new(headline),
                TextFont {
                    font: asset_server.load("BAUHS93.TTF").into(),
                    font_size: FontSize::Px(60.0),
                    ..default()
                },
                TextColor(GOLD.into()),
                Node {
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(hint),
                TextFont {
                    font: asset_server.load("BAUHS93.TTF").into(),
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                TextColor(GOLD.into()),
                Node {
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));
        })
    ;
}
