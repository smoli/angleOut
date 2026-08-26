---
id: c0004
title: Asset-driven level loading and hot reload
status: done
epic: e01
depends: [c0003]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T21:55:15
usage-tokens: 103523
usage-cost: 12.90396
---

## What

Load levels through Bevy's asset server instead of reading them once at startup,
so hand edits to a level file appear in a running game.

This means a `LevelAsset` plus a custom `AssetLoader`, and `Levels` becoming
handle-driven rather than owning `Vec<LevelDefinition>` — which touches every
`get_current_level()` caller.

## Acceptance criteria

- [x] A `LevelAsset` and its `AssetLoader` load `assets/levels/*.ron` through the asset server.
- [x] `Levels` holds handles rather than owned `LevelDefinition` values.
- [x] Every `get_current_level()` / `get_current_level_mut()` caller is migrated and behaves as before (`arena`, `level`, `events`, `pickups`, `match`).
- [x] Editing a level file on disk while the game is running updates it without a restart.
- [x] Systems that read the current level tolerate it not being loaded yet, rather than panicking on the first frames.
- [x] Playing every level still behaves exactly as before.
- [x] `cargo build` is clean and `cargo test` passes.

## Notes

- Real wrinkle: `distribute_global_pickups` currently mutates the level through
  `get_current_level_mut()` at spawn time. Once levels are shared assets, that
  becomes a mutation of shared data. Decide whether that derived per-match state
  moves to `MatchState` instead — likely yes.
- This is the card most likely to sprawl; it is kept separate from `c0003` for
  exactly that reason.

### How it came out

- `LevelAsset` is a newtype over `LevelDefinition` rather than an
  `impl Asset for LevelDefinition`, so "the level on disk" stays a different
  thing from "the level the editor edits and saves" - `c0012` writes a
  `LevelDefinition`, and only the asset server ever makes a `LevelAsset`.
- `Levels` is `Vec<Handle<LevelAsset>>` plus `current_level`.
  `get_current_level(&Assets<LevelAsset>)` returns an `Option`, so nothing
  unwraps; `get_current_level_mut` is gone entirely, which is what forced the
  pickup question below.
- **The mutation the card warned about moved.**
  `distributed_global_pickups` is no longer a `#[serde(skip)]` field on
  `LevelDefinition` - it is per-match state on `MatchState`, derived from the
  level's authored `global_pickups` by
  `MatchState::distribute_global_pickups`. `MatchState::reset()` clears it, and
  `pickup_spawn_globals_on_event` no longer needs `Levels` at all. Two matches
  of the same level can no longer fight over where its pickups landed.
- **Entering a match now waits.** Levels arrive asynchronously, so
  `game_flow_handler` holds the `StartMatch` transition (a `Local<bool>`, not a
  re-sent message - a reader and a writer of the same message type in one system
  is a parameter conflict) until `Levels::readiness` says `Ready`. That is what
  keeps "plays exactly as before" true rather than merely likely; without it a
  first match could start in front of a level that had not arrived.
- `LevelReadiness::Unavailable` - a campaign entry that is missing or will not
  parse - falls back to the menu with an error rather than entering an
  unplayable match. That case used to be a startup panic; now that it can also
  be a typo made in a text editor mid-game, the menu is recoverable: fix the
  file, press start, the asset server has already re-read it.
- **Hot reload replaces the blocks of the running match**, not the whole match.
  `level_reload` watches `AssetEvent::Modified` for the current level's handle,
  despawns every `Block` and re-runs the spawn. The arena, the ship and the
  score carry on. Reloading the background or the obstacles would mean tearing
  down and rebuilding the 3D environment mid-match, which belongs with the
  playtest round trip in `c0013`. There is one frame of latency, because
  `Assets` flushes its queued events in `PostUpdate`.
- `Levels` is inserted in `LevelPlugin::build`, not from a `Startup` system:
  `bevy_state` puts the initial `StateTransition` *before* `PreStartup`, so
  `OnEnter(GameState::InGame)` - which reads `Levels` - has already run by the
  time any startup schedule gets a turn. Loading is still asynchronous; only the
  handles exist that early. The campaign index itself is still read with
  `std::fs` and still panics naming the file, since play order is not something
  a running game re-reads.
- The loader declares `extensions() = ["ron"]` even though nothing needs it to:
  the asset server resolves a loader by asset type first, so that list is not
  what routes a level here. It matters only when a reload of a typed handle
  fails - the server then retries the path untyped, and with nothing registered
  that retry logged `Could not find an asset loader matching` on top of the real
  parse error. Verified both ways against the running game.
- `bevy` gained the `file_watcher` feature, which is what turns a hand edit into
  an `AssetEvent::Modified` at all.

### How it was checked

- `cargo test`: 53 passed, 0 failed. New coverage: the asset server loading the
  campaign the files spell (compared field for field against `c0003`'s legacy
  literals), a hand edit replacing the blocks of a running match, an edit to a
  level nobody is playing being ignored *and the same edit to the played level
  landing*, a match started before the level arrived spawning nothing instead of
  panicking, and readiness going `Loading -> Ready` for a real file and
  `Loading -> Unavailable` for one that does not exist.
- End to end in the real game, not just in tests: with a temporary auto-start
  system (removed again) the game entered a match on `level0`, and adding a row
  of nine blocks to `assets/levels/level0.ron` in a text editor logged
  `Level 0 changed on disk - reloaded it with 15 blocks` - 6 before, 6 + 9
  after. Writing garbage into a level file reported the parse error naming
  `LevelAssetLoader`, and restoring the file reloaded it cleanly.
- What was *not* checked by playing: this session cannot send input to the
  window, so "every level behaves exactly as before" rests on the campaign
  loading field-for-field equal to the pre-migration literals plus the block
  counts above, not on playing all seven through.

### Known limitation

If the level `GameState::NextLevel` moves to cannot be loaded, the fall back to
the menu restarts the campaign at level 0 rather than resuming. Only reachable
by breaking a level file mid-run.

## Review

### 2026-08-26T00:18:37 — pass

Checked: every acceptance criterion, the diff in `b81e65c`, `cargo build`,
`cargo test`, and the file watcher against the running binary.

- `LevelAsset` + `LevelAssetLoader` (`src/level/asset.rs`) load `levels/*.ron`
  through the asset server, and `the_asset_server_loads_the_campaign_the_files_spell`
  compares what comes back out of `Assets<LevelAsset>` field for field against
  `c0003`'s legacy literals.
- `Levels` is `Vec<Handle<LevelAsset>>` (`src/level/mod.rs`); `load_levels` takes
  an `&AssetServer` and the resource is inserted in `LevelPlugin::build`, which is
  after `DefaultPlugins` in `main`, so the `AssetServer` lookup there is sound.
- Every pre-migration caller is migrated: `git grep get_current_level 2bd8b33`
  names `arena`, `events`, `level` and `pickups`, and each now takes
  `Res<Assets<LevelAsset>>` and copes with `None`; `r#match` only ever used
  `next_level()`. `get_current_level_mut` is gone from the tree.
- Hot reload verified against the running game, not only in tests: with the built
  binary running I appended a line to `assets/levels/level0.ron` and
  `bevy_asset::server` logged `Reloaded levels/level0.ron` (file restored
  afterwards, `git status` clean for it). `notify` is in the dependency tree, so
  the `file_watcher` feature is really on. The half downstream of the watcher is
  covered by `a_hand_edit_to_the_current_level_replaces_its_blocks` and by
  `editing_a_level_that_is_not_being_played_leaves_the_match_alone`, whose second
  half proves the handle filter rather than nothing working. `block_spawn`
  augments the request entity instead of replacing it, so `level_reload`'s
  `Query<Entity, With<Block>>` does reach fully spawned blocks too.
- Nothing unwraps a level that has not arrived: `arena_spawn`, `level_spawn` and
  `match_event_handler` all early-return, and `game_flow_handler` holds
  `StartMatch` on `Levels::readiness`, whose three outcomes each have a test.
- The pickup mutation moved without changing play: `MatchState::reset()` clears
  `distributed_global_pickups`, but `match_spawn` is ordered
  `.before(SystemLabels::UpdateWorld)` and `level_spawn` sits in that set, so the
  distribution is made after the reset rather than being wiped by it, and
  `pickup_spawn_globals_on_event` keys off the same remaining-block counts as
  before.
- `cargo build` is clean; the 20 warnings are pre-existing bar one this card
  causes — `function load_level is never used` in `src/level/campaign.rs`, now
  only reachable from tests. `cargo test`: 53 passed, 0 failed, 0 ignored, both
  with and without `CARGO_MANIFEST_DIR` pointed at the repo. No test was removed,
  skipped or loosened; the campaign identity test got stricter, since the legacy
  comparison now runs against both the `std::fs` read and the asset-server read.
- The diff stays inside the What — `Cargo.toml`, `arena`, `events`, `level/*`,
  `main`, `match/state`, `pickups` — and the Bevy 0.19 migration edits carried in
  `src/events/mod.rs` are the same carry-along `c0002` and `c0003` made for the
  files they touched. No debug leftovers, no auto-start system left behind, no
  level file modified by the live verification.
- "Every level behaves exactly as before" is met as far as it can be verified
  here: campaign equality field for field plus a caller-by-caller read of the
  migration. Playing all seven through needs input this session cannot send
  either, which is what the card already records.

Observation for a follow-up card, not blocking: `level_reload` despawns and
respawns blocks without touching `TriggerStates`, so reloading a level that uses
trigger groups mid-match keeps the old blocks' `consumed`/`state` entries for
those groups.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
- 2026-08-26 `LevelAsset` + loader, `Levels` handle-driven, pickup distribution
  moved to `MatchState`, match start gated on the level being loaded, blocks
  reloaded in-match on a hand edit; `cargo test` 53 passed, verified live in the
  running game (agent)
- 2026-08-26 status → review (agent)
- 2026-08-26 status → done (app)
