extern crate core;

use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::gltf::Gltf;
use bevy::prelude::{AssetServer, ClearColor, Color, Commands, Handle, Msaa, PluginGroup, Res, Resource, Vec3, WindowDescriptor, WindowMode};
use bevy::utils::default;
use bevy::window::{close_on_esc, MonitorSelection, WindowPlugin, WindowPosition};
use bevy_framepace::FramepacePlugin;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::actions::{CameraActions, GameFlowActions, MatchActions};
use crate::arena::ArenaPlugin;
use crate::ball::BallPlugin;
use crate::block::BlockPlugin;
use crate::config::{ARENA_HEIGHT, ARENA_WIDTH_H, BLOCK_GAP, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::events::EventsPlugin;
use crate::game::GamePlugin;
use crate::level::{LevelDefinition, LevelObstacle, LevelPlugin, Levels, TargetLayout, WinCriteria};
use crate::level::TargetLayout::Custom;
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
        D = 1 only top
        Z = unbreakable

        Z is used for obstacles and do not count as blocks when determining of the
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
        I - Portal - Use this as a trigger target. Teleports the ball from the trigger to itself, preserving momentum

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
"AA
 AA
 AA
 AF
 AE
 AA";

const LEVEL0: &str =
"ZIR1 .. .. .. CA .. .. .. ZAA1
 ..   .. .. .. CA .. .. .. ..
 ..   .. .. .. CA .. .. .. ..
 ..   .. .. .. CA .. .. .. ..
 ..   .. .. .. CA .. .. .. ..
 ..   .. .. .. CA .. .. .. ..
";

const LEVEL1: &str =
"AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA";

const LEVEL2: &str =
"AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA
 AA AA AA AA AA AA AA AA AA";

const LEVEL3: &str =
"AA AA AA AA AA AA AA AA AA
 BA BA BA BA BA BA BA BA AA
 AA AA AA AA AA AA AA AA AA
 BA BA BA BA BA BA BA BA AA
 AA AA AA AA AA AA AA AA AA
 BA BA BA BA BA BA BA BA AA";


const LEVEL4: &str =
"BA BA BA BA CA BA BA BA BA
 AA AA AA AA CA AA AA AA AA
 BA BA BA BA CA BA BA BA BA
 ZA AA AA AA CA AA AA AA ZA
 ZA BA BA BA CA BA BA BA ZA
 ZA ZA ZA ZA ZA ZA ZA ZA ZA";


// For 011_Factory
const LEVEL5: &str =
"AA AA AA AA AA .. AA AA AA AA AA
 AA AA AA AA AA .. AA AA AA AA AA
 AA AA AA AA AA .. AA AA AA AA AA
 AA AA AA AA AA .. AA AA AA AA AA
 AA AA AA AA AA .. AA AA AA AA AA
 AA AA AA AA AA .. AA AA AA AA AA";

// For 012_Factory
const LEVEL6: &str =
"AA AA AA .. AA AA AA .. AA AA AA
 AA AA AA .. AA AA AA .. AA AA AA
 AA AA AA .. AA AA AA .. AA AA AA
 AA AA AA .. AA AA AA .. AA AA AA
 AA AA AA .. AA AA AA .. AA AA AA
 AA AA AA .. AA AA AA .. AA AA AA";






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
    // app.add_plugin(FramepacePlugin);
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
            targets: TargetLayout::SparseGrid(LEVEL0.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1)],
            ..default()
        },


/*        LevelDefinition {
            background_asset: "ship3_003.glb#Scene13".to_string(),
            simultaneous_balls: 1,
            targets: Custom("Conveyor".to_string()),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            obstacles: vec![
                LevelObstacle::ForceField(Vec3::new(100.0, 0.0, (-18.39 - 48.39) / 2.0), Vec3::NEG_X, (48.39 - 18.39), true),
                LevelObstacle::DirectionalDeathTrigger(Vec3::new(160.0, 0.0, (-18.39 - 48.39) / 2.0), Vec3::NEG_X, (48.39 - 18.39)),

                LevelObstacle::ForceField(Vec3::new(-100.0, 0.0, (-18.39 - 48.39) / 2.0), Vec3::X, (48.39 - 18.39), false),
                LevelObstacle::DirectionalDeathTrigger(Vec3::new(-160.0, 0.0, (-18.39 - 48.39) / 2.0), Vec3::X, (48.39 - 18.39)),

                LevelObstacle::Box(Vec3::new(-ARENA_WIDTH_H - 20.0, 0.0, 100.0 - 18.95), 40.0, 200.0),
                LevelObstacle::Box(Vec3::new(-ARENA_WIDTH_H - 20.0, 0.0, -48.39 - 50.0), 40.0, 100.0),
                LevelObstacle::Box(Vec3::new(ARENA_WIDTH_H + 20.0, 0.0, 100.0 - 18.95), 40.0, 200.0),
                LevelObstacle::Box(Vec3::new(ARENA_WIDTH_H + 20.0, 0.0, -48.39 - 50.0), 40.0, 100.0),
            ],
            default_wall_l: false,
            default_wall_r: false,
            win_criteria: WinCriteria::BlockHitPercentage(0.5),
            ..default()
        },*/

        LevelDefinition {
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL1.to_string(), BLOCK_GAP),
            time_limit: None,
            background_scroll_velocity: 20.0,
            global_pickups: vec![PickupType::MoreBalls(1)],
            ..default()
        },

        LevelDefinition {
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL2.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            ..default()
        },

        LevelDefinition {
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL3.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            ..default()
        },

        LevelDefinition {
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL4.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            ..default()
        },

        LevelDefinition {
            background_asset: "ship3_003.glb#Scene11".to_string(),
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL5.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            obstacles: vec![
                LevelObstacle::Box(Vec3::new(0.0, 0.0, -70.0), 15.0, 200.0),
            ],
            ..default()
        },

        LevelDefinition {
            background_asset: "ship3_003.glb#Scene12".to_string(),
            simultaneous_balls: 1,
            targets: TargetLayout::SparseGrid(LEVEL6.to_string(), BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            obstacles: vec![
                LevelObstacle::Box(Vec3::new(34.0, 0.0, -70.0), 15.0, 200.0),
                LevelObstacle::Box(Vec3::new(-34.0, 0.0, -70.0), 15.0, 200.0),

            ],
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


