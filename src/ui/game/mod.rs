use bevy::app::{App, Plugin, Update};
use bevy::color::palettes::css::{ANTIQUE_WHITE, GOLD, RED};
use bevy::prelude::{default, in_state, AlignSelf, AssetServer, Color, Commands, Component, Entity, FlexDirection, GamepadButton, IntoScheduleConfigs, JustifyContent, KeyCode, MessageReader, MessageWriter, Node, OnEnter, OnExit, Query, Res, Text, TextColor, TextFont, Transform, Val, With};
use bevy::text::FontSize;
use bevy::ui::{BackgroundColor, UiRect};
use bevy::world_serialization::WorldAssetRoot;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::prelude::ActionState;

use crate::events::GameFlowEvent;
use crate::state::GameState;
use crate::ui::{UIAction, UIEvents};

#[derive(PartialEq, Copy, Clone, Debug)]
enum OptionValues {
    NewGame,
    Editor,
    Settings,
}


impl TryFrom<u8> for OptionValues {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NewGame),
            1 => Ok(Self::Editor),
            2 => Ok(Self::Settings),

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
            .add_systems(OnEnter(GameState::InGame), (ui_spawn, ship_spawn))

            .add_systems(
                Update,
                (ui_handle_action, ui_update).run_if(in_state(GameState::InGame)),
            )

            .add_systems(OnExit(GameState::InGame), ui_despawn)
        ;
    }
}


fn ship_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        WorldAssetRoot(asset_server.load("ship3_003.glb#Scene4")),
        Transform::from_xyz(-15.0, 5.0, 0.0),
        UITag,
    ));
}


fn ui_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_events: MessageWriter<UIEvents>,
) {
    let font = TextFont {
        font: asset_server.load("BAUHS93.TTF").into(),
        font_size: FontSize::Px(60.0),
        ..default()
    };

    let centered = Node {
        align_self: AlignSelf::FlexStart,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                margin: UiRect {
                    left: Val::Percent(10.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        ))
        .insert(UIState {
            selected: OptionValues::NewGame,
        })


        .with_children(|parent| {
            parent.spawn((
                Text::new("New Game"),
                font.clone(),
                TextColor(GOLD.into()),
                centered.clone(),
                SelectOptions {
                    value: OptionValues::NewGame
                },
            ));

            parent.spawn((
                Text::new("Editor"),
                font.clone(),
                TextColor(GOLD.into()),
                centered.clone(),
                SelectOptions {
                    value: OptionValues::Editor
                },
            ));

            parent.spawn((
                Text::new("Settings"),
                font.clone(),
                TextColor(GOLD.into()),
                centered.clone(),
                SelectOptions {
                    value: OptionValues::Settings
                },
            ));
        })
        .insert(UITag)
        .insert(
            InputMap::default()
                .with(UIAction::ActivateSelection, GamepadButton::South)
                .with(UIAction::ActivateSelection, KeyCode::Space)
                .with(UIAction::SelectDown, GamepadButton::DPadDown)
                .with(UIAction::SelectUp, GamepadButton::DPadUp),
        )
    ;

    // Trigger render
    ui_events.write(UIEvents::SelectionChange);
}


fn ui_handle_action(
    mut actions: Query<(&mut UIState, &ActionState<UIAction>)>,
    mut ui_events: MessageWriter<UIEvents>,
) {
    for (mut state, action) in &mut actions {
        let mut curr = state.selected as u8;
        if action.just_pressed(&UIAction::SelectDown) {
            curr += 1;
        }

        if action.just_pressed(&UIAction::SelectUp) && curr > 0 {
            curr -= 1;
        }

        match OptionValues::try_from(curr) {
            Ok(v) => state.selected = v.clone(),
            Err(_) => {}
        }

        ui_events.write(UIEvents::SelectionChange);

        if action.just_pressed(&UIAction::ActivateSelection) {
            ui_events.write(UIEvents::SelectionActivated(state.selected as u8));
        }
    }
}


fn ui_update(
    mut ui_events: MessageReader<UIEvents>,
    mut options: Query<(&mut TextColor, &SelectOptions)>,
    ui: Query<&UIState>,
    mut game_event: MessageWriter<GameFlowEvent>,
) {
    for ev in ui_events.read() {
        match ev {
            UIEvents::SelectionChange => {
                let Ok(ui_state) = ui.single() else { continue; };

                for (mut color, option) in &mut options {
                    if option.value == ui_state.selected {
                        color.0 = RED.into();
                    } else {
                        color.0 = ANTIQUE_WHITE.into();
                    }
                }
            }

            UIEvents::SelectionActivated(num) => {
                let o = OptionValues::try_from(*num).expect("Unknown value");


                //info!("Player chose {:?}", o);

                match o {
                    OptionValues::NewGame => { game_event.write(GameFlowEvent::StartMatch); }

                    OptionValues::Editor => { game_event.write(GameFlowEvent::OpenEditor); }

                    OptionValues::Settings => {}
                }
            }
        }
    }
}

fn ui_despawn(mut commands: Commands, uis: Query<Entity, With<UITag>>) {
    //info!("Despawning game Screen");
    for ui in &uis {
        //info!("Despawn game ui {:?}", ui);
        commands.entity(ui).despawn();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use bevy::app::{App, Update};
    use bevy::ecs::message::Messages;
    use bevy::MinimalPlugins;

    /// `ui_handle_action` walks the menu by casting the selection to `u8` and
    /// stepping the number, so the discriminants and `TryFrom` have to agree.
    /// Adding an entry to one and not the other makes an item unreachable
    /// without breaking the build.
    #[test]
    fn every_menu_entry_is_reachable_by_walking_the_menu() {
        let entries = [OptionValues::NewGame, OptionValues::Editor, OptionValues::Settings];

        for entry in entries {
            assert_eq!(OptionValues::try_from(entry as u8), Ok(entry));
        }

        assert_eq!(OptionValues::try_from(entries.len() as u8), Err(()));
    }

    /// The menu with nothing spawned in it - `ui_update`'s activation branch
    /// only needs the two message queues.
    fn menu_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<UIEvents>();
        app.add_message::<GameFlowEvent>();
        app.add_systems(Update, ui_update);
        app
    }

    fn activate(app: &mut App, option: OptionValues) -> Vec<GameFlowEvent> {
        app.world_mut()
            .resource_mut::<Messages<UIEvents>>()
            .write(UIEvents::SelectionActivated(option as u8));

        app.update();

        app.world_mut()
            .resource_mut::<Messages<GameFlowEvent>>()
            .drain()
            .collect()
    }

    #[test]
    fn picking_the_editor_entry_asks_for_the_editor() {
        let mut app = menu_app();

        let events = activate(&mut app, OptionValues::Editor);

        assert!(
            matches!(events.as_slice(), [GameFlowEvent::OpenEditor]),
            "expected the editor to be opened, got {events:?}"
        );
    }

    /// ... and the entry above it still starts a game, so the new item did not
    /// just shift the menu one down.
    #[test]
    fn picking_new_game_still_starts_a_match() {
        let mut app = menu_app();

        let events = activate(&mut app, OptionValues::NewGame);

        assert!(
            matches!(events.as_slice(), [GameFlowEvent::StartMatch]),
            "expected a match to start, got {events:?}"
        );
    }
}
