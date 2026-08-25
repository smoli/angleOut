---
id: c0004
title: Asset-driven level loading and hot reload
status: ready
epic: e01
depends: [c0003]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:59:37
order: 40
---

## What

Load levels through Bevy's asset server instead of reading them once at startup,
so hand edits to a level file appear in a running game.

This means a `LevelAsset` plus a custom `AssetLoader`, and `Levels` becoming
handle-driven rather than owning `Vec<LevelDefinition>` — which touches every
`get_current_level()` caller.

## Acceptance criteria

- [ ] A `LevelAsset` and its `AssetLoader` load `assets/levels/*.ron` through the asset server.
- [ ] `Levels` holds handles rather than owned `LevelDefinition` values.
- [ ] Every `get_current_level()` / `get_current_level_mut()` caller is migrated and behaves as before (`arena`, `level`, `events`, `pickups`, `match`).
- [ ] Editing a level file on disk while the game is running updates it without a restart.
- [ ] Systems that read the current level tolerate it not being loaded yet, rather than panicking on the first frames.
- [ ] Playing every level still behaves exactly as before.
- [ ] `cargo build` is clean and `cargo test` passes.

## Notes

- Real wrinkle: `distribute_global_pickups` currently mutates the level through
  `get_current_level_mut()` at spawn time. Once levels are shared assets, that
  becomes a mutation of shared data. Decide whether that derived per-match state
  moves to `MatchState` instead — likely yes.
- This is the card most likely to sprawl; it is kept separate from `c0003` for
  exactly that reason.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
