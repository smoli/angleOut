use bevy::app::App;
use bevy::asset::AssetServer;
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::log::info;
use bevy::prelude::{Plugin, Res, Commands, NodeBundle, AlignSelf, Size, Color, TextBundle, TextStyle, SystemSet, Component, Query, Entity, With, FlexWrap, GamepadButtonType, EventWriter};
use bevy::text::TextSection;
use bevy::ui::{FlexDirection, JustifyContent, Style, Val, ZIndex};
use bevy::utils::default;
use leafwing_input_manager::InputManagerBundle;
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
            .add_system_set(
                SystemSet::on_enter(GameState::PostMatch)
                    .with_system(ui_spawn)
            )

            .add_system_set(
                SystemSet::on_update(GameState::PostMatch)
                    .with_system(ui_handle_action)
            )

            .add_system_set(
                SystemSet::on_exit(GameState::PostMatch)
                    .with_system(ui_despawn)
            )
        ;
    }
}

fn ui_despawn(
    mut commands: Commands,
    ui: Query<Entity, With<UITag>>
) {
    for ui in &ui {
        commands.entity(ui)
            .despawn_recursive();
    }
}

fn ui_handle_action(
    mut actions: Query<&mut ActionState<GameFlowActions>, With<UITag>>,
    mut game_event: EventWriter<GameFlowEvent>,
) {
    for mut action in &mut actions {
        if action.just_released(GameFlowActions::StartMatch) {
            info!("Player requested Start!");
            action.consume(GameFlowActions::StartMatch);
            game_event.send(GameFlowEvent::StartMatch);
        }
    }
}

fn ui_spawn(
    player: Res<Player>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands
        .spawn(NodeBundle {
            node: Default::default(),
            style: Style {
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.2).into(),
            z_index: ZIndex::Global(100),
            ..default()
        })

        .insert(UITag)
        .insert(InputManagerBundle::<GameFlowActions> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(GamepadButtonType::South, GameFlowActions::StartMatch)
                .build(),
        })

        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections(
                [
                    TextSection::new(
                        match player.state {
                            PlayerState::Open => "You shouldn't be here!",
                            PlayerState::HasWon => "You won!",
                            PlayerState::HasLost => "You loose!"
                        },
                        TextStyle {
                            font: asset_server.load("BAUHS93.TTF"),
                            font_size: 60.0,
                            color: Color::GOLD,
                        }
                    ),
                    TextSection::new(
                        "Press A/X to play again",
                        TextStyle {
                            font: asset_server.load("BAUHS93.TTF"),
                            font_size: 30.0,
                            color: Color::GOLD,
                        }
                    )
                ]
            ).with_style(Style {
                align_self: AlignSelf::Center,
                flex_wrap: FlexWrap::Wrap,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..default()
            })


            )
            ;
        })
    ;
}