//! Reading levels off disk.
//!
//! A level is one `assets/levels/*.ron` file holding a whole
//! [`LevelDefinition`]; `assets/levels/campaign.ron` lists the ones that make up
//! the game, in play order. Any other level file in the directory is scratch -
//! it stays loadable and testable without being part of the campaign.
//!
//! The campaign index is read once at startup with `std::fs` - the play order is
//! not something a running game re-reads. The levels themselves go through the
//! asset server, so a hand edit to a level file hot-reloads into a running game
//! (see [`crate::level::asset`]).

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::asset::io::file::FileAssetReader;
use bevy::asset::AssetServer;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::level::asset::LevelAsset;
use crate::level::{LevelDefinition, Levels};

/// Where the level files live, relative to the asset root.
pub const LEVELS_DIR: &str = "assets/levels";

/// The same directory as an asset path, which is rooted at `assets/` already.
pub const LEVELS_ASSET_DIR: &str = "levels";

/// The campaign index, inside [`LEVELS_DIR`].
pub const CAMPAIGN_FILE: &str = "campaign.ron";

/// The play order: level file names, relative to the directory the campaign
/// itself sits in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Campaign {
    pub levels: Vec<String>,
}

/// The level directory of the running game.
///
/// Resolved through Bevy's own asset root so `std::fs` here and the asset server
/// always look at the same files - including when `BEVY_ASSET_ROOT` or
/// `CARGO_MANIFEST_DIR` moves the root.
pub fn levels_dir() -> PathBuf {
    FileAssetReader::get_base_path().join(LEVELS_DIR)
}

/// The asset path of a level file named by the campaign index.
pub fn level_asset_path(name: &str) -> String {
    format!("{LEVELS_ASSET_DIR}/{name}")
}

#[derive(Debug)]
pub enum LevelLoadError {
    Io(PathBuf, std::io::Error),
    Parse(PathBuf, ron::error::SpannedError),
}

impl fmt::Display for LevelLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelLoadError::Io(path, err) => write!(f, "{}: {}", path.display(), err),
            LevelLoadError::Parse(path, err) => write!(f, "{}: {}", path.display(), err),
        }
    }
}

fn read(path: &Path) -> Result<String, LevelLoadError> {
    fs::read_to_string(path).map_err(|e| LevelLoadError::Io(path.to_path_buf(), e))
}

/// How a level is written back out.
///
/// `escape_strings(false)` is what keeps the block layout a readable ASCII map:
/// RON falls back to a raw string for it instead of one long line of `\n`s.
fn pretty_config() -> PrettyConfig {
    PrettyConfig::new()
        .escape_strings(false)
        .struct_names(true)
}

pub fn parse_level(source: &str) -> Result<LevelDefinition, ron::error::SpannedError> {
    ron::from_str(source)
}

pub fn level_to_ron(level: &LevelDefinition) -> Result<String, ron::Error> {
    ron::ser::to_string_pretty(level, pretty_config())
}

pub fn load_level(path: &Path) -> Result<LevelDefinition, LevelLoadError> {
    parse_level(&read(path)?).map_err(|e| LevelLoadError::Parse(path.to_path_buf(), e))
}

pub fn load_campaign(dir: &Path) -> Result<Campaign, LevelLoadError> {
    let path = dir.join(CAMPAIGN_FILE);
    let source = read(&path)?;

    ron::from_str(&source).map_err(|e| LevelLoadError::Parse(path, e))
}

/// The campaign as the game plays it: a handle per level the index names, in
/// order.
///
/// The index has to exist and parse - a campaign we cannot even name is a
/// startup error. The level files behind it are then loaded asynchronously, so a
/// broken one surfaces later, as an asset load failure.
pub fn load_levels(dir: &Path, asset_server: &AssetServer) -> Result<Levels, LevelLoadError> {
    let campaign = load_campaign(dir)?;

    Ok(Levels {
        handles: campaign
            .levels
            .iter()
            .map(|name| asset_server.load::<LevelAsset>(level_asset_path(name)))
            .collect(),
        current_level: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::app::App;
    use bevy::asset::{AssetApp, AssetPlugin, Assets};
    use bevy::prelude::{default, Vec3};
    use bevy::MinimalPlugins;

    use crate::config::{ARENA_WIDTH_H, BLOCK_GAP};
    use crate::level::asset::LevelAssetLoader;
    use crate::r#match::state::MatchState;
    use crate::level::{LevelObstacle, TargetLayout, WinCriteria};
    use crate::pickups::PickupType;

    /// The repo's own level directory, independent of whatever asset root the
    /// environment points at.
    fn dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(LEVELS_DIR)
    }

    fn level_files() -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = fs::read_dir(dir())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "ron"))
            .filter(|p| p.file_name().unwrap() != CAMPAIGN_FILE)
            .collect();
        files.sort();
        files
    }

    // The levels exactly as they read in `main.rs` before they became files.
    // Kept here so the migration can be proved rather than eyeballed.

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

    const SIMPLE1: &str =
"AA
 AA
 AA
 AF
 AE
 AA";

    const DEMO_MOVING: &str = "AA AA AA AA AA AA AA
 AA .. .. AE .. .. AA
 AA AG .. .. .. AH AA
 AA .. .. AF .. .. AA
 AA CA CA CA CA CA AA";

    const DEMO_MINIMAL_WIN_STATE_ERROR: &str = "AA AH AA";

    fn legacy_campaign() -> Vec<LevelDefinition> {
        vec![
            LevelDefinition {
                simultaneous_balls: 1,
                targets: TargetLayout::SparseGrid(LEVEL0.to_string(), BLOCK_GAP),
                time_limit: None,
                global_pickups: vec![PickupType::MoreBalls(1)],
                ..default()
            },
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
                global_pickups: vec![
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                ],
                ..default()
            },
            LevelDefinition {
                simultaneous_balls: 1,
                targets: TargetLayout::SparseGrid(LEVEL4.to_string(), BLOCK_GAP),
                time_limit: None,
                global_pickups: vec![
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                ],
                ..default()
            },
            LevelDefinition {
                background_asset: "ship3_003.glb#Scene11".to_string(),
                simultaneous_balls: 1,
                targets: TargetLayout::SparseGrid(LEVEL5.to_string(), BLOCK_GAP),
                time_limit: None,
                global_pickups: vec![
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                ],
                obstacles: vec![LevelObstacle::Box(Vec3::new(0.0, 0.0, -70.0), 15.0, 200.0)],
                ..default()
            },
            LevelDefinition {
                background_asset: "ship3_003.glb#Scene12".to_string(),
                simultaneous_balls: 1,
                targets: TargetLayout::SparseGrid(LEVEL6.to_string(), BLOCK_GAP),
                time_limit: None,
                global_pickups: vec![
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                    PickupType::MoreBalls(1),
                ],
                obstacles: vec![
                    LevelObstacle::Box(Vec3::new(34.0, 0.0, -70.0), 15.0, 200.0),
                    LevelObstacle::Box(Vec3::new(-34.0, 0.0, -70.0), 15.0, 200.0),
                ],
                ..default()
            },
        ]
    }

    /// The commented-out force field level from `main.rs`. Scratch, not part of
    /// the campaign - `c0001` needs it to check the force field impact mapping.
    fn legacy_conveyor() -> LevelDefinition {
        LevelDefinition {
            background_asset: "ship3_003.glb#Scene13".to_string(),
            simultaneous_balls: 1,
            targets: TargetLayout::Custom("Conveyor".to_string()),
            time_limit: None,
            global_pickups: vec![
                PickupType::MoreBalls(1),
                PickupType::MoreBalls(1),
                PickupType::MoreBalls(1),
            ],
            obstacles: vec![
                LevelObstacle::ForceField(
                    Vec3::new(100.0, 0.0, (-18.39 - 48.39) / 2.0),
                    Vec3::NEG_X,
                    48.39 - 18.39,
                    true,
                ),
                LevelObstacle::DirectionalDeathTrigger(
                    Vec3::new(160.0, 0.0, (-18.39 - 48.39) / 2.0),
                    Vec3::NEG_X,
                    48.39 - 18.39,
                ),
                LevelObstacle::ForceField(
                    Vec3::new(-100.0, 0.0, (-18.39 - 48.39) / 2.0),
                    Vec3::X,
                    48.39 - 18.39,
                    false,
                ),
                LevelObstacle::DirectionalDeathTrigger(
                    Vec3::new(-160.0, 0.0, (-18.39 - 48.39) / 2.0),
                    Vec3::X,
                    48.39 - 18.39,
                ),
                LevelObstacle::Box(
                    Vec3::new(-ARENA_WIDTH_H - 20.0, 0.0, 100.0 - 18.95),
                    40.0,
                    200.0,
                ),
                LevelObstacle::Box(
                    Vec3::new(-ARENA_WIDTH_H - 20.0, 0.0, -48.39 - 50.0),
                    40.0,
                    100.0,
                ),
                LevelObstacle::Box(
                    Vec3::new(ARENA_WIDTH_H + 20.0, 0.0, 100.0 - 18.95),
                    40.0,
                    200.0,
                ),
                LevelObstacle::Box(
                    Vec3::new(ARENA_WIDTH_H + 20.0, 0.0, -48.39 - 50.0),
                    40.0,
                    100.0,
                ),
            ],
            default_wall_l: false,
            default_wall_r: false,
            win_criteria: WinCriteria::BlockHitPercentage(0.5),
            ..default()
        }
    }

    /// The campaign as the files on disk spell it, read without an asset server.
    fn campaign_definitions() -> Vec<LevelDefinition> {
        load_campaign(&dir())
            .unwrap()
            .levels
            .iter()
            .map(|name| load_level(&dir().join(name)).unwrap())
            .collect()
    }

    #[test]
    fn the_campaign_is_the_levels_that_used_to_live_in_main() {
        assert_eq!(campaign_definitions(), legacy_campaign());
    }

    #[test]
    fn the_campaign_index_becomes_one_handle_per_level_in_order() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<LevelAsset>()
            .register_asset_loader(LevelAssetLoader);

        let asset_server = app.world().resource::<AssetServer>().clone();
        let levels = load_levels(&dir(), &asset_server).unwrap();

        assert_eq!(levels.current_level, 0);
        assert_eq!(levels.handles.len(), load_campaign(&dir()).unwrap().levels.len());

        for (handle, name) in levels.handles.iter().zip(load_campaign(&dir()).unwrap().levels) {
            assert_eq!(
                handle.path().map(|p| p.path().to_string_lossy().to_string()),
                Some(level_asset_path(&name)),
                "{name}"
            );
        }
    }

    /// The asset server, its loader and the files line up: what a match reads
    /// out of `Assets<LevelAsset>` is the campaign, not something adjacent.
    #[test]
    fn the_asset_server_loads_the_campaign_the_files_spell() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<LevelAsset>()
            .register_asset_loader(LevelAssetLoader);

        let asset_server = app.world().resource::<AssetServer>().clone();
        let levels = load_levels(&dir(), &asset_server).unwrap();

        for _ in 0..2000 {
            if levels
                .handles
                .iter()
                .all(|h| asset_server.is_loaded_with_dependencies(h))
            {
                break;
            }
            app.update();
        }

        let assets = app.world().resource::<Assets<LevelAsset>>();
        let loaded: Vec<LevelDefinition> = levels
            .handles
            .iter()
            .map(|h| {
                assets
                    .get(h)
                    .unwrap_or_else(|| panic!("{:?} never loaded", h.path()))
                    .0
                    .clone()
            })
            .collect();

        assert_eq!(loaded, legacy_campaign());
    }

    #[test]
    fn the_scratch_levels_survived_the_move_too() {
        assert_eq!(load_level(&dir().join("conveyor.ron")).unwrap(), legacy_conveyor());

        for (file, layout) in [
            ("simple1.ron", SIMPLE1),
            ("demo_moving.ron", DEMO_MOVING),
            ("demo_minimal_win_state_error.ron", DEMO_MINIMAL_WIN_STATE_ERROR),
        ] {
            let level = load_level(&dir().join(file)).unwrap();

            assert_eq!(
                level.targets,
                TargetLayout::SparseGrid(layout.to_string(), BLOCK_GAP),
                "{file}"
            );
        }
    }

    #[test]
    fn scratch_levels_are_not_part_of_the_campaign() {
        let campaign = load_campaign(&dir()).unwrap();

        for scratch in [
            "conveyor.ron",
            "simple1.ron",
            "demo_moving.ron",
            "demo_minimal_win_state_error.ron",
        ] {
            assert!(
                !campaign.levels.contains(&scratch.to_string()),
                "{scratch} should be scratch, not campaign"
            );
        }
    }

    #[test]
    fn every_campaign_entry_names_a_level_that_exists() {
        for name in load_campaign(&dir()).unwrap().levels {
            let path = dir().join(&name);
            assert!(path.is_file(), "{} is missing", path.display());
        }
    }

    #[test]
    fn every_level_file_round_trips() {
        for path in level_files() {
            let once = load_level(&path).unwrap();
            let written = level_to_ron(&once).unwrap();
            let twice = parse_level(&written).unwrap_or_else(|e| {
                panic!("re-reading written {}: {e}\n{written}", path.display())
            });

            assert_eq!(once, twice, "{}", path.display());
            assert_eq!(
                written,
                level_to_ron(&twice).unwrap(),
                "{} does not write back the same",
                path.display()
            );
        }
    }

    #[test]
    fn a_written_level_keeps_its_layout_a_readable_ascii_map() {
        let written = level_to_ron(&load_level(&dir().join("level4.ron")).unwrap()).unwrap();

        assert!(
            written.contains("ZA ZA ZA ZA ZA ZA ZA ZA ZA"),
            "the block map should survive writing as lines, got:\n{written}"
        );
    }

    /// Where a level's pickups land is per-match state on `MatchState`, derived
    /// from the authored `global_pickups` - so it is neither read from nor
    /// written to a level file.
    #[test]
    fn distributed_pickups_are_derived_rather_than_read() {
        let level = load_level(&dir().join("level0.ron")).unwrap();
        assert_eq!(level.global_pickups, vec![PickupType::MoreBalls(1)]);

        let mut state = MatchState::default();
        assert!(state.distributed_global_pickups.is_empty());

        state.distribute_global_pickups(&level.global_pickups, 24);
        assert_eq!(state.distributed_global_pickups.len(), 1);

        let written = level_to_ron(&level).unwrap();
        assert!(!written.contains("distributed_global_pickups"), "{written}");
    }
}
