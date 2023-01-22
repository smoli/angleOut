use bevy::log::info;
use bevy::prelude::{App, AssetServer, BackgroundColor, BuildChildren, Color, Commands, Component, default, DespawnRecursiveExt, Entity, EventReader, EventWriter, FlexDirection, GamepadButtonType, JustifyContent, NodeBundle, Plugin, Query, Res, SceneBundle, Size, Style, SystemSet, Text, TextBundle, TextSection, TextStyle, Transform, TransformBundle, Val, With};
use bevy::ui::{AlignSelf, UiRect};
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::ActionState;

use crate::events::GameFlowEvent;
use crate::state::GameState;
use crate::ui::{UIAction, UIEvents};

#[derive(PartialEq, Copy, Clone, Debug)]
enum OptionValues {
    NewGame,
    Settings,
}


impl TryFrom<u8> for OptionValues {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NewGame),
            1 => Ok(Self::Settings),

            _ => Err(())
        }
    }
}

#[derive(Component)]
struct UIState {
    selected: OptionValues,
}


#[derive(Component)]
struct SelectOptions {
    value: OptionValues,
}

#[derive(Component)]
struct UITag;

pub struct UIGamePlugin;

impl Plugin for UIGamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(ui_spawn)
                    .with_system(ship_spawn)
            )

            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(ui_handle_action)
                    .with_system(ui_update)
            )

            .add_system_set(
                SystemSet::on_exit(GameState::InGame)
                    .with_system(ui_despawn)
            )
        ;
    }
}


fn ship_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("ship3_003.glb#Scene4"),
        ..default()
    })
        .insert(TransformBundle::from_transform(Transform::from_xyz(-15.0, 5.0, 0.0)))
        .insert(UITag)

    ;
}


fn ui_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_events: EventWriter<UIEvents>,
) {
    let style = TextStyle {
        font: asset_server.load("BAUHS93.TTF"),
        font_size: 60.0,
        color: Color::GOLD,
    };

    let centered = Style {
        align_self: AlignSelf::FlexStart,
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                margin: UiRect {
                    left: Val::Percent(10.0),
                    ..default()
                },
                ..default()
            },
            background_color: BackgroundColor::from(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        })
        .insert(UIState {
            selected: OptionValues::NewGame,
        })


        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    "New Game", style.clone(),
                )
            ])
                .with_style(centered.clone())
            )
                .insert(SelectOptions {
                    value: OptionValues::NewGame
                });

            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    "Settings", style.clone(),
                )
            ])
                .with_style(centered.clone())
            )
                .insert(SelectOptions {
                    value: OptionValues::Settings
                });
        })
        .insert(UITag)
        .insert(InputManagerBundle::<UIAction> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(GamepadButtonType::South, UIAction::ActivateSelection)
                .insert(GamepadButtonType::DPadDown, UIAction::SelectDown)
                .insert(GamepadButtonType::DPadUp, UIAction::SelectUp)
                .build(),
        })
    ;

    // Trigger render
    ui_events.send(UIEvents::SelectionChange);
}


fn ui_handle_action(
    mut actions: Query<(&mut UIState, &mut ActionState<UIAction>)>,
    mut ui_events: EventWriter<UIEvents>,
) {
    for (mut state, mut action) in &mut actions {
        let mut curr = state.selected as u8;
        if action.just_pressed(UIAction::SelectDown) {
            curr += 1;
            action.consume(UIAction::SelectDown);
        }

        if action.just_pressed(UIAction::SelectUp) && curr > 0 {
            curr -= 1;
            action.consume(UIAction::SelectUp);
        }

        match OptionValues::try_from(curr) {
            Ok(v) => state.selected = v.clone(),
            Err(_) => {}
        }

        ui_events.send(UIEvents::SelectionChange);

        if action.just_pressed(UIAction::ActivateSelection) {
            ui_events.send(UIEvents::SelectionActivated(state.selected as u8));
            action.consume(UIAction::ActivateSelection);
        }
    }
}


fn ui_update(
    mut ui_events: EventReader<UIEvents>,
    mut options: Query<(&mut Text, &SelectOptions)>,
    ui: Query<&UIState>,
    mut game_event: EventWriter<GameFlowEvent>,
) {
    for ev in ui_events.iter() {
        match ev {
            UIEvents::SelectionChange => {
                let ui_state = ui.get_single().unwrap();

                for (mut text, option) in &mut options {
                    if option.value == ui_state.selected {
                        for mut section in &mut text.sections {
                            section.style.color = Color::RED
                        }
                    } else {
                        for mut section in &mut text.sections {
                            section.style.color = Color::ANTIQUE_WHITE
                        }
                    }
                }
            }

            UIEvents::SelectionActivated(num) => {
                let o = OptionValues::try_from(*num).expect("Unknown value");


                info!("Player chose {:?}", o);

                match o {
                    OptionValues::NewGame => game_event.send(GameFlowEvent::StartMatch),

                    OptionValues::Settings => {}
                }
            }
        }
    }
}

fn ui_despawn(mut commands: Commands, uis: Query<Entity, With<UITag>>) {
    info!("Despawning game Screen");
    for ui in &uis {
        commands.entity(ui).despawn_recursive();
    }
}