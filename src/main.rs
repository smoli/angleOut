extern crate core;

use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::gltf::Gltf;
use bevy::prelude::{AssetServer, ClearColor, Color, Commands, Handle, Msaa, PluginGroup, Res, Resource, WindowDescriptor, WindowMode};
use bevy::utils::default;
use bevy::window::{close_on_esc, MonitorSelection, WindowPlugin, WindowPosition};
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::actions::{CameraActions, GameFlowActions, MatchActions};
use crate::arena::ArenaPlugin;
use crate::ball::BallPlugin;
use crate::block::BlockPlugin;
use crate::config::{BLOCK_GAP, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::events::EventsPlugin;
use crate::game::GamePlugin;
use crate::level::{LevelDefinition, LevelPlugin, Levels, TargetLayout};
use crate::particles::ParticlePlugin;
use crate::physics::PhysicsPlugin;
use crate::pickups::{PickupsPlugin, PickupType};
use crate::player::PlayerPlugin;
use crate::points::PointsPlugin;
use crate::r#match::MatchPlugin;
use crate::ship::ShipPlugin;
use crate::state::GameState;
use crate::ui::UI;

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
mod materials;
mod points;
mod particles;
mod powerups;
mod pickups;

const DEMO_MOVING: &str = "AA AA AA AA AA AA AA
 AA .. .. AE .. .. AA
 AA AG .. .. .. AH AA
 AA .. .. AF .. .. AA
 AA CA CA CA CA CA AA";


const DEMO_MINIMAL_WIN_STATE_ERROR: &str = "AA AH AA";


/// Helper resource for tracking our asset
#[derive(Resource)]
struct MyAssetPack(Handle<Gltf>);

fn main() {
    let mut app = App::new();

    // app.insert_resource(Msaa { samples: 4 });


    app.add_system(load_gltf);

    setup_screen(&mut app);
    setup_ui(&mut app);
    app.add_plugin(EventsPlugin);

    app.add_state(GameState::InMatch);

    app.add_plugin(PhysicsPlugin);

    app.add_plugin(ShipPlugin);
    app.add_plugin(ArenaPlugin);
    app.add_plugin(BallPlugin);
    app.add_plugin(BlockPlugin);
    app.add_plugin(LevelPlugin);
    app.add_plugin(GamePlugin);
    app.add_plugin(MatchPlugin);
    app.add_plugin(PointsPlugin);
    app.add_plugin(ParticlePlugin);
    app.add_plugin(PickupsPlugin);
    app.add_plugin(PlayerPlugin);

    app.add_plugin(InputManagerPlugin::<GameFlowActions>::default());
    app.add_plugin(InputManagerPlugin::<MatchActions>::default());
    app.add_plugin(InputManagerPlugin::<CameraActions>::default());

    app.add_system(close_on_esc);


    let levelDefinitions: Vec<LevelDefinition> = vec![
        LevelDefinition {
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(DEMO_MOVING.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            ..default()
        }
    ];


    app.insert_resource(Levels {
        definitions: levelDefinitions,
        current_level: 0,
    });


   /* app.insert_resource(LevelDefinition {
        simultaneous_balls: 1,
        // targets: TargetLayout::FilledGrid(10, 5, BlockType::Simple, BlockBehaviour::SittingDuck, BLOCK_GAP),
        targets: TargetLayout::SparseGrid(DEMO_MOVING.to_string(), BLOCK_GAP),
        time_limit: None,
        global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
        ..default()
    });
*/
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
            window: WindowDescriptor {
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
                position: WindowPosition::Centered,
                monitor: MonitorSelection::Current,
                // mode: WindowMode::SizedFullscreen,
                title: "Angle Out".to_string(),
                cursor_visible: false,
                ..default()
            },
            ..default()
        }));
}


fn setup_ui(app: &mut App) {
    app.add_plugin(UI);
}


