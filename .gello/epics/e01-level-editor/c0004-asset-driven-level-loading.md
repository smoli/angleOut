---
id: c0004
title: Asset-driven level loading and hot reload
status: review
epic: e01
depends: [c0003]
created: 2026-08-25
updated: 2026-08-26
status-changed: 2026-08-26T00:12:48
usage-tokens: 78780
usage-cost: 10.5086
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

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
- 2026-08-26 `LevelAsset` + loader, `Levels` handle-driven, pickup distribution
  moved to `MatchState`, match start gated on the level being loaded, blocks
  reloaded in-match on a hand edit; `cargo test` 53 passed, verified live in the
  running game (agent)
- 2026-08-26 status → review (agent)
