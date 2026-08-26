use bevy::app::{App, Plugin, Update};
use bevy::color::palettes::css::GOLD;
use bevy::prelude::{default, in_state, AlignSelf, AssetServer, Color, Commands, Component, Entity, GamepadButton, IntoScheduleConfigs, MessageWriter, Node, OnEnter, OnExit, Query, Res, Text, TextColor, TextFont, Val, With};
use bevy::text::FontSize;
use bevy::ui::BackgroundColor;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::prelude::ActionState;
use crate::actions::GameFlowActions;
use crate::events::GameFlowEvent;
use crate::state::GameState;


#[derive(Component)]
struct UITag;

pub struct UIStartPlugin;

impl Plugin for UIStartPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Start), ui_spawn)

            .add_systems(Update, ui_handle_action.run_if(in_state(GameState::Start)))

            .add_systems(OnExit(GameState::Start), ui_despawn)
        ;
    }
}


fn ui_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_self: AlignSelf::Center,

                ..default()
            },
            BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Angle Out - Press A to play"),
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
        })
        .insert(UITag)
        .insert(
            InputMap::default()
                .with(GameFlowActions::StartGame, GamepadButton::South),
        )
    ;
}


fn ui_handle_action(
    actions: Query<&ActionState<GameFlowActions>, With<UITag>>,
    mut game_event: MessageWriter<GameFlowEvent>,
) {
    for action in &actions {
        if action.just_released(&GameFlowActions::StartGame) {
            //info!("Player requested Start!");
            game_event.write(GameFlowEvent::StartGame);
        }
    }
}


fn ui_despawn(mut commands: Commands, uis: Query<Entity, With<UITag>>) {
    //info!("Despawning Start Screen");
    for ui in &uis {
        commands.entity(ui).despawn();
    }
}
