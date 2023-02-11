extern crate core;

use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
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



/*
    Each 2 to 4 character tuple describes one block

    1st Character
        How many hits can a block take:
        A = 1,
        B = 2,
        C = 3,
        Z = unbreakable

        Z are used for obstacles and do not count as blocks when determining of the
        player has finished the level

    2nd Character
        What behaviour does the block have
        A - Nothing
        B - Spinner - which is kinda useless I guess
        C - Vanisher - questionable as well
        D - Repulsor
        E - Evader first movement to the right
        F - Evader first movement to the left
        G - Evader first movement up
        H - Evader first movement down

    3rd Character (optional)
        Triggertype:
        A - Start Trigger
        B - Stop Trigger
        C - StartStop Trigger
        R - Receiver that starts stopped
        S - Receiver that starts started

        4th Character (mandatory if char 3 exists)
        Triggergroup 0..=9

 */

const SIMPLE1: &str =
"AA";

const LEVEL0: &str =
"ZA ZA ZA
 ZA AA ZA
 AA ZER1 AA
.. CAC1 ..";

const LEVEL1: &str = 
"AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA";


const LEVEL2: &str = 
"BA BA BA BA BA BA BA BA BA
 AA AA AA AA AA AA AA AA AA
 BA BA BA BA BA BA BA BA BA
 AA AA AA AA AA AA AA AA AA
 BA BA BA BA BA BA BA BA BA
 AA AA AA AA AA AA AA AA AA";



const LEVEL3: &str =
"BA BA BA BA BA BA BA BA BA
 AA AA AA AA BA AA AA AA AA
 BA BA BA BA BA BA BA BA BA
 ZA AA AA AA BA AA AA AA ZA
 ZA BA BA BA BA BA BA BA ZA
 ZA ZA ZA ZA ZA ZA ZA ZA ZA";



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

    app.add_state(GameState::InGame);

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
            targets: TargetLayout::SparseGrid(SIMPLE1.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1)],
            ..default()
        },

        LevelDefinition {
            background_asset: "ship3_003.glb#Scene11".to_string(),
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL2.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            ..default()
        },

        LevelDefinition {
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL3. to_string(), BLOCK_GAP),
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
        .add_plugin(FrameTimeDiagnosticsPlugin)
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


