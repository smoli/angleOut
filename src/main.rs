mod config;
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

use std::f32::consts::PI;
use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::math::Quat;
use bevy::pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle};
use bevy::prelude::{Camera, Camera3dBundle, ClearColor, Color, Commands, GamepadButtonType, OrthographicProjection, PluginGroup, Query, Transform, Vec3, WindowDescriptor, With};
use bevy::utils::default;
use bevy::window::{close_on_esc, MonitorSelection, WindowPlugin, WindowPosition};
use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::{ActionState, InputManagerPlugin, InputMap};
use crate::actions::{CameraActions, GameFlowActions, MatchActions};
use crate::arena::ArenaPlugin;
use crate::ball::BallPlugin;
use crate::block::{BlockBehaviour, BlockPlugin, BlockType};
use crate::config::{BLOCK_DEPTH, BLOCK_GAP, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::events::EventsPlugin;
use crate::game::GamePlugin;
use crate::level::{LevelDefinition, LevelPlugin, TargetLayout};
use crate::physics::PhysicsPlugin;
use crate::r#match::MatchPlugin;
use crate::ship::ShipPlugin;
use crate::state::GameState;
use crate::ui::UI;

fn main() {
    let mut app = App::new();

    setup_screen(&mut app);
    setup_ui(&mut app);
    app.add_plugin(EventsPlugin);

    app.add_state(GameState::Start);

    app.add_plugin(PhysicsPlugin);

    app.add_plugin(ShipPlugin);
    app.add_plugin(ArenaPlugin);
    app.add_plugin(BallPlugin);
    app.add_plugin(BlockPlugin);
    app.add_plugin(LevelPlugin);
    app.add_plugin(GamePlugin);
    app.add_plugin(MatchPlugin);

    app.add_plugin(InputManagerPlugin::<GameFlowActions>::default());
    app.add_plugin(InputManagerPlugin::<MatchActions>::default());
    app.add_plugin(InputManagerPlugin::<CameraActions>::default());

    app.add_system(close_on_esc);

    app.insert_resource(LevelDefinition {
        simultaneous_balls: 1,
        // targets: TargetLayout::FilledGrid(10, 5, BlockType::Simple, BlockBehaviour::Vanisher, BLOCK_GAP),
        targets: TargetLayout::SparseGrid(
".. .. .. .. ..
 .. CD AA CD ..
 . .. .. .. ..
 .. .. .. .. ..".to_string(), 5, BLOCK_GAP
        ),
        time_limit: None,
    }
    );

    app.run();
}

fn setup_screen(app: &mut App) {
    // app.insert_resource(ClearColor(Color::BLACK))
    app.insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            position: WindowPosition::Centered,
            monitor: MonitorSelection::Current,
            title: "Angle Out".to_string(),
            ..default()
        },
        ..default()
    }));
}


fn setup_ui(app: &mut App) {
    app.add_plugin(UI);
}


