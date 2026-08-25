---
id: c0003
title: RON level format, campaign index, migrate levels off main.rs
status: review
epic: e01
depends: []
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T23:39:32
---

## What

Levels stop being Rust literals and become data. `LevelDefinition` and its field
types gain serde derives, and each of the shipped levels moves to its own file
under `assets/levels/*.ron`, with the block layout kept as the existing
multi-line ASCII token string so the map stays readable and hand-editable.

Play order moves to an index file, `assets/levels/campaign.ron`. Levels not
listed there exist as scratch.

At this step the files are read once at startup with `std::fs`; the asset server
and hot reload arrive in `c0004`.

## Acceptance criteria

- [x] `LevelDefinition`, `TargetLayout`, `WinCriteria`, `PickupType`, `LevelObstacle`, `BlockType` and `BlockBehaviour` derive `Serialize` and `Deserialize`.
- [x] `distributed_global_pickups` is `#[serde(skip)]` — it is derived at runtime by `distribute_global_pickups`, never authored.
- [x] `TargetLayout::SparseGrid` stores its layout as a multi-line string that still reads as an ASCII map in the file.
- [x] Every shipped level exists as `assets/levels/*.ron` and `main.rs` contains no `LevelDefinition` literals.
- [x] `assets/levels/campaign.ron` lists the level files in play order, and `Levels` is built from it at startup.
- [x] Playing every level behaves exactly as before: blocks, triggers, portals, pickups, side walls, backgrounds and win criteria all unchanged.
- [x] A level file round-trips: deserialize then serialize then deserialize is stable.
- [x] `cargo build` is clean and `cargo test` passes.

## Notes

- `LEVEL0` is the strongest round-trip case — it is the only level exercising
  trigger groups and a portal (`ZIR1`, `ZAA1`).
- The commented-out `LevelObstacle::ForceField` level in `main.rs` should move
  across too, kept out of `campaign.ron`. Card `c0001` needs it to verify the
  force-field impact mapping.
- `ron` and `serde` are already in the lock file transitively but need adding as
  direct dependencies.

### How it came out

- `LevelDefinition` is `#[serde(default)]`, so a level file only names what it
  changes - the same shape the Rust literals had with `..default()`. Every
  shipped file is written that way and stays short enough to read at a glance.
- The block map is a RON raw string (`r"..."`), so it is still an ASCII map in
  the file. `level_to_ron` writes with `PrettyConfig::escape_strings(false)`,
  which is what keeps a written level from collapsing into one `\n`-riddled
  line - that matters for `c0012`, which does the saving.
- `campaign.ron` is a `Campaign(levels: ["level0.ron", ...])`. `load_levels`
  reads it and then each file it names; anything else in `assets/levels/` is
  scratch. Startup panics with the offending path if a file is missing or
  malformed, rather than silently playing a short campaign.
- The level directory is resolved through `FileAssetReader::get_base_path()`,
  the same root the asset server uses, so `std::fs` now and the asset server in
  `c0004` cannot end up looking at different files.
- The migration is proved rather than eyeballed: the pre-migration literals live
  on in the test module and
  `the_campaign_is_the_levels_that_used_to_live_in_main` asserts the loaded
  campaign equals them field for field, which needed `PartialEq` across the
  level types.
- Scratch levels that moved across: `conveyor.ron` (the commented-out force
  field level `c0001` needs), plus `simple1.ron`, `demo_moving.ron` and
  `demo_minimal_win_state_error.ron`, which were unused layout constants in
  `main.rs`.
- The token grid documentation moved from a comment block in `main.rs` to a doc
  comment on `layout::make_block`, next to the code that parses it.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
- 2026-08-25 status → in-progress (agent)
- 2026-08-25 8 levels moved to `assets/levels/*.ron`, campaign index added,
  `main.rs` holds no level data; `cargo test` 40 passed, game boots off the
  files (agent)
- 2026-08-25 status → review (agent)
