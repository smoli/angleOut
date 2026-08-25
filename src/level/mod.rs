use std::time::Duration;

use bevy::app::{App, Plugin, Update};
use bevy::asset::{AssetApp, AssetEvent, AssetId, AssetServer, Assets, Handle};
use bevy::log::{error, info, warn_once};
use bevy::math::Vec2;
use bevy::prelude::{default, in_state, Commands, Component, Entity, IntoScheduleConfigs, MessageReader, OnEnter, Query, Res, ResMut, Resource, Vec3, With};
use serde::{Deserialize, Serialize};

use crate::block::{Block, BlockBehaviour, BlockType};
use crate::config::{ARENA_WIDTH_H, BLOCK_GAP, BLOCK_WIDTH};
use crate::labels::SystemLabels;
use crate::level::asset::{LevelAsset, LevelAssetLoader};
use crate::level::layout::{generate_block_grid, interpret_grid};
use crate::level::TargetLayout::{FilledGrid, SparseGrid};
use crate::pickups::PickupType;
use crate::r#match::state::MatchState;
use crate::ship::Ship;
use crate::state::GameState;

pub mod asset;
pub mod campaign;
mod layout;

#[derive(Component)]
pub struct RequestTag;


/// How a level's blocks are laid out.
///
/// `SparseGrid` carries the block map as the multi-line ASCII token string
/// documented in `layout::make_block`, so a level file stays readable and
/// hand-editable; the `f32` is the gap between cells.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetLayout {
    FilledGrid(usize, usize, BlockType, BlockBehaviour, f32),
    SparseGrid(String, f32),
    Custom(String)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LevelObstacle {
    // Center position, width, height
    Box(Vec3, f32, f32),

    // Center position, Normal, width, flip normal when rotating in place (hacky)
    ForceField(Vec3, Vec3, f32, bool),

    DirectionalDeathTrigger(Vec3, Vec3, f32)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WinCriteria {
    BlockHitPercentage(f32)
}

/// One level, as it is authored in `assets/levels/*.ron`.
///
/// Every field defaults, so a level file only has to name what it changes from
/// [`LevelDefinition::default`] - the same shape the hardcoded levels had with
/// `..default()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LevelDefinition {
    pub background_asset: String,
    pub background_scroll_velocity: f32,
    pub simultaneous_balls: i32,
    pub win_criteria: WinCriteria,
    pub targets: TargetLayout,
    pub time_limit: Option<Duration>,

    /// The pickups this level hands out over the course of a match. Where each
    /// one lands is derived per match by
    /// [`MatchState::distribute_global_pickups`], not authored here.
    pub global_pickups: Vec<PickupType>,
    pub obstacles: Vec<LevelObstacle>,
    pub default_wall_l: bool,
    pub default_wall_r: bool,

}

impl Default for LevelDefinition {
    fn default() -> Self {
        return LevelDefinition {
            background_asset: "ship3_003.glb#Scene10".to_string(),
            background_scroll_velocity: 0.0,
            simultaneous_balls: 1,
            win_criteria: WinCriteria::BlockHitPercentage(1.0),
            targets: FilledGrid(5, 5, BlockType::Simple, BlockBehaviour::SittingDuck, BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![],
            obstacles: vec![],
            default_wall_l: true,
            default_wall_r: true
        }
    }
}


/// The campaign: one asset handle per level, plus where in it we are.
///
/// Handles rather than owned [`LevelDefinition`]s, so the level a match is
/// playing is the same value the asset server owns - and a hand edit to the file
/// reaches the running game instead of a copy taken at startup.
#[derive(Resource)]
pub struct Levels {
    pub handles: Vec<Handle<LevelAsset>>,
    pub current_level: usize
}

/// Whether the current level can be played yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelReadiness {
    Ready,

    /// The asset server is still reading the file. Worth waiting for.
    Loading,

    /// There is nothing left to wait for: the campaign is empty, or the file is
    /// missing or will not parse.
    Unavailable,
}

impl Levels {
    pub fn current_handle(&self) -> Option<&Handle<LevelAsset>> {
        self.handles.get(self.current_level)
    }

    /// The level being played, if the asset server has it.
    ///
    /// `None` while the file is still in flight, which is why every caller has to
    /// cope rather than unwrap.
    pub fn get_current_level<'a>(&self, levels: &'a Assets<LevelAsset>) -> Option<&'a LevelDefinition> {
        levels.get(self.current_handle()?).map(|level| &level.0)
    }

    pub fn is_current_level(&self, id: AssetId<LevelAsset>) -> bool {
        self.current_handle().is_some_and(|handle| handle.id() == id)
    }

    pub fn readiness(&self, levels: &Assets<LevelAsset>, asset_server: &AssetServer) -> LevelReadiness {
        let Some(handle) = self.current_handle() else {
            return LevelReadiness::Unavailable;
        };

        if levels.contains(handle) {
            LevelReadiness::Ready
        } else if asset_server.load_state(handle).is_loading() {
            LevelReadiness::Loading
        } else {
            LevelReadiness::Unavailable
        }
    }

    pub fn next_level(&mut self) -> bool {
        if self.current_level + 1 < self.handles.len() {
            self.current_level += 1;
            true
        } else {
            false
        }

    }
}


pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<LevelAsset>()
            .register_asset_loader(LevelAssetLoader)

            .add_systems(
                OnEnter(GameState::InMatch),
                level_spawn.in_set(SystemLabels::UpdateWorld),
            )

            .add_systems(
                Update,
                level_reload
                    .in_set(SystemLabels::UpdateWorld)
                    .run_if(in_state(GameState::InMatch)),
            )

        ;

        app.insert_resource(levels(app));
    }
}

/// Hands the campaign index to the asset server.
///
/// Done at plugin build time rather than from a `Startup` system because the
/// initial `StateTransition` runs *before* `PreStartup` - so
/// `OnEnter(GameState::InGame)`, which reads `Levels`, has already happened by
/// the time any startup schedule gets a turn. Loading stays asynchronous: this
/// asks the asset server for the files, it does not wait for them.
///
/// The campaign index itself has to be there - a game that cannot name its
/// levels has nothing to play - so a missing or malformed `campaign.ron` is a
/// startup panic naming the file, as it was before the levels became assets.
fn levels(app: &App) -> Levels {
    let asset_server = app.world().resource::<AssetServer>().clone();

    campaign::load_levels(&campaign::levels_dir(), &asset_server)
        .unwrap_or_else(|e| panic!("could not load the campaign: {e}"))
}



fn make_filled_grid(
    commands: &mut Commands,
    cols: usize, rows: usize, block_type: &BlockType, behaviour: &BlockBehaviour, gap: f32) -> i32
{
    let positions = generate_block_grid(rows, cols, gap);

    for i in 0..positions.len() {
        let pos = positions.get(i).unwrap();

        commands.
            spawn(Block {
                position: pos.clone(),
                behaviour: behaviour.clone(),
                block_type: block_type.clone(),
                ..default()
            })
            .insert(RequestTag);
    }

    positions.len() as i32
}

fn make_grid_from_string_layout(
    commands: &mut Commands,
    layout: &String,
    gap: f32,
) -> i32 {
    if let Some(res) = interpret_grid(layout, gap) {
        let mut c = 0;
        for b in res {
            if b.block_type != BlockType::Obstacle {
                c += 1;
            }

            commands
                .spawn(b)
                .insert(RequestTag);

        }

        return c;
    }

    0
}

/// Spawns the level's blocks and tells the match how many there are.
///
/// Split out of [`level_spawn`] because [`level_reload`] needs exactly this and
/// nothing else - a hot reload replaces the blocks, it does not restart the
/// match or respawn the ship.
fn spawn_targets(commands: &mut Commands, level: &LevelDefinition, stats: &mut MatchState) {
    match &level.targets {
        FilledGrid(cols, rows, block_type, behaviour, gap) => {
            let count = make_filled_grid(commands, *cols, *rows, block_type, behaviour, *gap);
            stats.distribute_global_pickups(&level.global_pickups, count as usize);
            stats.set_block_count(count);

        }

        SparseGrid(layout, gap) => {
            let count = make_grid_from_string_layout(commands, layout, *gap);
            stats.distribute_global_pickups(&level.global_pickups, count as usize);
            stats.set_block_count(count);
        }

        TargetLayout::Custom(name) => {

            match name.as_str() {
                "Conveyor" => {
                    level_span_conveyor(stats, commands);
                }

                _ => {
                    error!("Unknown custom level definition {}", name);
                }
            };

        }
    };
}

fn level_spawn(
    mut stats: ResMut<MatchState>,
    levels: Res<Levels>,
    level_assets: Res<Assets<LevelAsset>>,
    mut commands: Commands) {
    commands
        .spawn(Ship::default())
        .insert(RequestTag);

    let Some(level) = levels.get_current_level(&level_assets) else {
        warn_once!("Entered a match before level {} finished loading - no blocks spawned", levels.current_level);
        return;
    };

    spawn_targets(&mut commands, level, &mut stats);
}

/// Puts a hand edit to the current level file into the running match.
///
/// Bevy's file watcher re-runs the loader and swaps the new value in under the
/// same handle; all we have to do is replace the blocks that came from the old
/// one. Only the blocks - the arena, the ship and the score carry on, so editing
/// a level's map is a live operation rather than a restart.
fn level_reload(
    mut events: MessageReader<AssetEvent<LevelAsset>>,
    levels: Res<Levels>,
    level_assets: Res<Assets<LevelAsset>>,
    blocks: Query<Entity, With<Block>>,
    mut stats: ResMut<MatchState>,
    mut commands: Commands,
) {
    let reloaded = events.read().any(|event| match event {
        AssetEvent::Modified { id } => levels.is_current_level(*id),
        _ => false,
    });

    if !reloaded {
        return;
    }

    let Some(level) = levels.get_current_level(&level_assets) else { return; };

    for block in &blocks {
        commands.entity(block).despawn();
    }

    spawn_targets(&mut commands, level, &mut stats);

    info!("Level {} changed on disk - reloaded it with {} blocks", levels.current_level, stats.blocks);
}

fn level_span_conveyor(
    stats: &mut MatchState,
    commands: &mut Commands
) {
    let count_per_row = 2;
    let mut pos = Vec2::new(ARENA_WIDTH_H + 3.0, -25.0);
    for _i in 0..count_per_row {
        let speed = 10.0;
        commands.
            spawn(Block {
                position: pos.clone(),
                behaviour: BlockBehaviour::EvaderL(speed),
                block_type: BlockType::Simple,
                ..default()
            })
            .insert(RequestTag);
        pos.x += 2.0 * BLOCK_WIDTH + BLOCK_GAP;
    }


    let mut pos = Vec2::new(-ARENA_WIDTH_H - 3.0, -35.0);
    for _i in 0..count_per_row {
        let speed2 = 10.0;
        commands.
            spawn(Block {
                position: pos.clone(),
                behaviour: BlockBehaviour::EvaderR(speed2),
                block_type: BlockType::Simple,
                ..default()
            })
            .insert(RequestTag);
        pos.x -= 2.0 * BLOCK_WIDTH + BLOCK_GAP;
    }

    stats.clear_pickups();
    stats.set_block_count(2 * count_per_row);
}






#[cfg(test)]
mod tests {
    use super::*;

    use bevy::asset::AssetPlugin;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::MinimalPlugins;

    use crate::pickups::PickupType;

    fn sparse(layout: &str) -> LevelDefinition {
        LevelDefinition {
            targets: SparseGrid(layout.to_string(), BLOCK_GAP),
            ..default()
        }
    }

    /// Just enough app to run the level systems: the asset collection, the match
    /// state they write to, and `level_reload` in `Update`.
    fn level_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<LevelAsset>();
        app.register_asset_loader(LevelAssetLoader);
        app.insert_resource(MatchState::default());
        app.add_systems(Update, level_reload);
        app
    }

    fn add_level(app: &mut App, level: LevelDefinition) -> Handle<LevelAsset> {
        app.world_mut()
            .resource_mut::<Assets<LevelAsset>>()
            .add(LevelAsset(level))
    }

    fn set_level(app: &mut App, handle: &Handle<LevelAsset>, level: LevelDefinition) {
        app.world_mut()
            .resource_mut::<Assets<LevelAsset>>()
            .get_mut(handle)
            .unwrap()
            .0 = level;
    }

    /// Two frames, because `Assets` flushes its queued `AssetEvent`s in
    /// `PostUpdate` - so an edit made now is only readable in `Update` next
    /// frame. That one frame of latency is what a hot reload costs.
    fn settle(app: &mut App) {
        app.update();
        app.update();
    }

    fn block_count(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut blocks = world.query::<&Block>();
        blocks.iter(world).count()
    }

    /// A match that has already spawned the current level, as `OnEnter(InMatch)`
    /// would have.
    fn start_match(app: &mut App) {
        app.world_mut().run_system_once(level_spawn).unwrap();
    }

    #[test]
    fn a_match_spawns_the_level_the_asset_server_is_holding() {
        let mut app = level_app();
        let handle = add_level(&mut app, sparse("AA AA AA"));
        app.insert_resource(Levels { handles: vec![handle], current_level: 0 });

        start_match(&mut app);

        assert_eq!(block_count(&mut app), 3);
        assert_eq!(app.world().resource::<MatchState>().blocks, 3);
    }

    /// The point of the whole card: edit the file, see it in the running match.
    /// Bevy's file watcher is what turns a hand edit into the `Modified` event;
    /// this covers everything downstream of it.
    #[test]
    fn a_hand_edit_to_the_current_level_replaces_its_blocks() {
        let mut app = level_app();
        let handle = add_level(&mut app, sparse("AA AA"));
        app.insert_resource(Levels { handles: vec![handle.clone()], current_level: 0 });
        start_match(&mut app);

        settle(&mut app);
        assert_eq!(block_count(&mut app), 2, "nothing changed, so nothing respawned");

        set_level(&mut app, &handle, sparse("AA AA AA AA"));
        settle(&mut app);

        assert_eq!(block_count(&mut app), 4, "the edited map should be the one on screen");
        assert_eq!(app.world().resource::<MatchState>().blocks, 4);
    }

    #[test]
    fn editing_a_level_that_is_not_being_played_leaves_the_match_alone() {
        let mut app = level_app();
        let playing = add_level(&mut app, sparse("AA AA"));
        let other = add_level(&mut app, sparse("AA AA AA AA AA"));
        app.insert_resource(Levels { handles: vec![playing.clone(), other.clone()], current_level: 0 });
        start_match(&mut app);

        set_level(&mut app, &other, sparse("AA"));
        settle(&mut app);
        assert_eq!(block_count(&mut app), 2, "a level nobody is playing must not touch the match");

        // ... and the same edit to the level being played does land, so the
        // assertion above is about the filter rather than about nothing working.
        set_level(&mut app, &playing, sparse("AA"));
        settle(&mut app);
        assert_eq!(block_count(&mut app), 1);
    }

    #[test]
    fn a_match_started_before_the_level_arrived_spawns_no_blocks_rather_than_panicking() {
        let mut app = level_app();
        app.insert_resource(Levels { handles: vec![Handle::default()], current_level: 0 });

        start_match(&mut app);
        settle(&mut app);

        assert_eq!(block_count(&mut app), 0);
    }

    #[test]
    fn a_level_is_ready_once_the_asset_server_has_it() {
        let mut app = level_app();
        let handle = add_level(&mut app, sparse("AA"));
        app.insert_resource(Levels { handles: vec![handle], current_level: 0 });

        assert_eq!(readiness(&app), LevelReadiness::Ready);
    }

    /// The readiness of whatever campaign the app is currently holding.
    fn readiness(app: &App) -> LevelReadiness {
        app.world().resource::<Levels>().readiness(
            app.world().resource::<Assets<LevelAsset>>(),
            app.world().resource::<AssetServer>(),
        )
    }

    /// Starting a match must wait for a level that is on its way ...
    #[test]
    fn a_level_still_in_flight_is_loading_rather_than_unavailable() {
        let mut app = level_app();
        let asset_server = app.world().resource::<AssetServer>().clone();
        app.insert_resource(Levels {
            handles: vec![asset_server.load(campaign::level_asset_path("level0.ron"))],
            current_level: 0,
        });

        assert_eq!(readiness(&app), LevelReadiness::Loading);

        for _ in 0..2000 {
            if readiness(&app) == LevelReadiness::Ready {
                break;
            }
            app.update();
        }

        assert_eq!(readiness(&app), LevelReadiness::Ready);
    }

    /// ... and give up on one that is never going to arrive, or a broken level
    /// file would hold the game at the menu forever.
    #[test]
    fn a_level_file_that_is_not_there_stops_being_loading() {
        let mut app = level_app();
        let asset_server = app.world().resource::<AssetServer>().clone();
        app.insert_resource(Levels {
            handles: vec![asset_server.load(campaign::level_asset_path("no-such-level.ron"))],
            current_level: 0,
        });

        for _ in 0..2000 {
            if readiness(&app) != LevelReadiness::Loading {
                break;
            }
            app.update();
        }

        assert_eq!(readiness(&app), LevelReadiness::Unavailable);
    }

    /// A campaign with nothing left to wait for must not hold the match forever.
    #[test]
    fn an_empty_campaign_is_unavailable_rather_than_loading() {
        let mut app = level_app();
        app.insert_resource(Levels { handles: vec![], current_level: 0 });

        assert_eq!(readiness(&app), LevelReadiness::Unavailable);
    }

    #[test]
    fn next_level_walks_the_campaign_and_stops_at_the_end() {
        let mut levels = Levels { handles: vec![Handle::default(), Handle::default()], current_level: 0 };

        assert!(levels.next_level());
        assert_eq!(levels.current_level, 1);
        assert!(!levels.next_level());
        assert_eq!(levels.current_level, 1);
    }

    #[test]
    fn next_level_on_an_empty_campaign_does_not_panic() {
        let mut levels = Levels { handles: vec![], current_level: 0 };

        assert!(!levels.next_level());
        assert_eq!(levels.current_level, 0);
    }

    /// Where the pickups land is per-match state now, so re-entering a level
    /// re-rolls them instead of replaying the first match's placement.
    #[test]
    fn every_spawn_of_a_level_gets_its_own_pickup_placement() {
        let mut app = level_app();
        let handle = add_level(&mut app, LevelDefinition {
            targets: SparseGrid("AA AA AA AA AA AA".to_string(), BLOCK_GAP),
            global_pickups: vec![PickupType::MoreBalls(1), PickupType::MoreBalls(1)],
            ..default()
        });
        app.insert_resource(Levels { handles: vec![handle.clone()], current_level: 0 });

        start_match(&mut app);
        let first = app.world().resource::<MatchState>().distributed_global_pickups.clone();
        assert_eq!(first.len(), 2);
        assert!(first.keys().all(|pos| *pos < 6));

        // The level asset itself never learned about any of it.
        let level = app.world().resource::<Assets<LevelAsset>>().get(&handle).unwrap();
        assert_eq!(level.global_pickups.len(), 2);
    }
}
