extern crate core;

use std::env;

use bevy::app::{App, Startup, Update};
use bevy::DefaultPlugins;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::gltf::Gltf;
use bevy::input::ButtonInput;
use bevy::state::app::AppExtStates;
use bevy::prelude::{default, in_state, not, AssetServer, ClearColor, Color, Commands, Entity, Handle, IntoScheduleConfigs, KeyCode, PluginGroup, Query, Res, Resource};
use bevy::window::{CursorOptions, MonitorSelection, Window, WindowPlugin, WindowPosition, WindowResolution};
#[allow(unused_imports)]
use bevy_framepace::FramepacePlugin;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::actions::{CameraActions, GameFlowActions, MatchActions};
use crate::arena::ArenaPlugin;
use crate::ball::BallPlugin;
use crate::block::BlockPlugin;
use crate::config::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::editor::EditorPlugin;
use crate::events::EventsPlugin;
use crate::game::GamePlugin;
use crate::input::InputDiagnosticsPlugin;
use crate::level::LevelPlugin;
use crate::particles::ParticlePlugin;
use crate::physics::PhysicsPlugin;
use crate::pickups::PickupsPlugin;
use crate::player::PlayerPlugin;
use crate::points::PointsPlugin;
use crate::r#match::MatchPlugin;
use crate::ship::ShipPlugin;
use crate::state::GameState;
use crate::ui::UI;

mod config;
mod editor;
mod r#match;
mod state;
mod events;
mod labels;
mod ui;
mod actions;
mod ship;
mod arena;
mod ball;
mod physics;
mod block;
mod level;
mod player;
mod game;
mod materials;
mod points;
mod particles;
mod powerups;
mod pickups;
mod input;



/// Helper resource for tracking our asset
#[derive(Resource)]
struct MyAssetPack(Handle<Gltf>);

fn main() {
    let mut app = App::new();


    env::set_var("RUST_BACKTRACE", "1");

    // Msaa is a per-camera component since Bevy 0.15:
    // camera.insert(Msaa::Sample4);


    setup_screen(&mut app);
    setup_ui(&mut app);
    app.add_systems(Startup, load_gltf);
    app.add_plugins(EventsPlugin);
    // app.add_plugins(FramepacePlugin);
    app.insert_state(GameState::InGame);

    app.add_plugins(PhysicsPlugin);

    app.add_plugins(ShipPlugin);
    app.add_plugins(ArenaPlugin);
    app.add_plugins(BallPlugin);
    app.add_plugins(BlockPlugin);
    app.add_plugins(LevelPlugin);
    app.add_plugins(EditorPlugin);
    app.add_plugins(GamePlugin);
    app.add_plugins(MatchPlugin);
    app.add_plugins(PointsPlugin);
    app.add_plugins(ParticlePlugin);
    app.add_plugins(PickupsPlugin);
    app.add_plugins(PlayerPlugin);
    app.add_plugins(InputDiagnosticsPlugin);

    app.add_plugins(InputManagerPlugin::<GameFlowActions>::default());
    app.add_plugins(InputManagerPlugin::<MatchActions>::default());
    app.add_plugins(InputManagerPlugin::<CameraActions>::default());

    // The editor uses `Escape` to get back to the menu, so quitting the game on
    // it would be two things on one key - see `crate::editor`.
    app.add_systems(Update, close_on_esc.run_if(not(in_state(GameState::Editor))));


    app.run();
}


fn load_gltf(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    let gltf = ass.load("ship3_003.glb");
    commands.insert_resource(MyAssetPack(gltf));
}

fn setup_screen(app: &mut App) {
    app
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),
                position: WindowPosition::Centered(MonitorSelection::Current),
                // mode: WindowMode::SizedFullscreen(MonitorSelection::Current),
                title: "Angle Out".to_string(),
                ..default()
            }),
            primary_cursor_options: Some(CursorOptions {
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default());
}


fn setup_ui(app: &mut App) {
    app.add_plugins(UI);
}

/// `bevy::window::close_on_esc` was removed from the engine, so we roll our own.
fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}


