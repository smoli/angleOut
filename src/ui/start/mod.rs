use bevy::log::info;
use bevy::prelude::{App, Color, Component, Commands, Entity, Plugin, Query, SystemSet, TextBundle, TextSection, TextStyle, With, AssetServer, Res, GamepadButtonType, EventWriter, NodeBundle, Style, Size, Val, default, BuildChildren, DespawnRecursiveExt};
use bevy::ui::{AlignSelf};
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::InputManagerBundle;
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
            .add_system_set(
                SystemSet::on_enter(GameState::Start)
                    .with_system(ui_spawn)
            )

            .add_system_set(
                SystemSet::on_update(GameState::Start)
                    .with_system(ui_handle_action)
            )

            .add_system_set(
                SystemSet::on_exit(GameState::Start)
                    .with_system(ui_despawn)
            )
        ;
    }
}


fn ui_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_self: AlignSelf::Center,

                ..default()

            },
            background_color: Color::rgb(0.65, 0.65, 0.65).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    "Angle Out - Press A to play",
                    TextStyle {
                        font: asset_server.load("BAUHS93.TTF"),
                        font_size: 60.0,
                        color: Color::GOLD,

                    },
                )
            ]).with_style(Style {
                align_self: AlignSelf::Center,
                ..default()
            })
            )
                ;
        })
        .insert(UITag)
        .insert(InputManagerBundle::<GameFlowActions> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(GamepadButtonType::South, GameFlowActions::StartGame)
                .build(),
        })
    ;
}


fn ui_handle_action(
    mut actions: Query<&mut ActionState<GameFlowActions>, With<UITag>>,
    mut game_event: EventWriter<GameFlowEvent>,
) {
    for mut action in &mut actions {
        if action.just_released(GameFlowActions::StartGame) {
            //info!("Player requested Start!");
            action.consume(GameFlowActions::StartGame);
            game_event.send(GameFlowEvent::StartGame);
        }
    }
}


fn ui_despawn(mut commands: Commands, uis: Query<Entity, With<UITag>>) {
    //info!("Despawning Start Screen");
    for ui in &uis {
        commands.entity(ui).despawn_recursive();
    }
}