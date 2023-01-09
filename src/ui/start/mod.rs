use bevy::asset::Handle;
use bevy::log::info;
use bevy::prelude::{AlignSelf, App, Color, Component, Commands, Entity, Plugin, PositionType, Query, SystemSet, TextBundle, TextSection, TextStyle, With, AssetServer, Res, Font, UiRect, Val, Text, GamepadButtonType, EventReader, EventWriter};
use bevy::ui::Style;
use bevy::utils::default;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::user_input::InputKind::GamepadButton;
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
    info!("Spawning Start Screen");
    commands.spawn((TextBundle::from_sections([
        TextSection::new(
            "Angle Out",
            TextStyle {
                font: asset_server.load("BAUHS93.TTF"),
                font_size: 60.0,
                color: Color::GOLD,

            },
        )
    ]),
                    UITag
    ))
        .insert(InputManagerBundle::<GameFlowActions> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(GamepadButtonType::South, GameFlowActions::StartMatch)
                .build(),
        });
}



fn ui_handle_action(
    mut actions: Query<&mut ActionState<GameFlowActions>, With<UITag>>,
    mut game_event: EventWriter<GameFlowEvent>
) {
    for mut action in &mut actions {
        if action.pressed(GameFlowActions::StartMatch) {
            info!("Player requested Start!");
            action.consume(GameFlowActions::StartMatch);
            game_event.send(GameFlowEvent::StartGame);
        }
    }
}


fn ui_despawn(mut commands: Commands, uis: Query<Entity, With<UITag>>) {
    info!("Despawning Start Screen");
    for ui in &uis {
        commands.entity(ui).despawn();
    }
}