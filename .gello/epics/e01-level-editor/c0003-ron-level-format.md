---
id: c0003
title: RON level format, campaign index, migrate levels off main.rs
status: backlog
epic: e01
depends: []
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:58:50
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

- [ ] `LevelDefinition`, `TargetLayout`, `WinCriteria`, `PickupType`, `LevelObstacle`, `BlockType` and `BlockBehaviour` derive `Serialize` and `Deserialize`.
- [ ] `distributed_global_pickups` is `#[serde(skip)]` — it is derived at runtime by `distribute_global_pickups`, never authored.
- [ ] `TargetLayout::SparseGrid` stores its layout as a multi-line string that still reads as an ASCII map in the file.
- [ ] Every shipped level exists as `assets/levels/*.ron` and `main.rs` contains no `LevelDefinition` literals.
- [ ] `assets/levels/campaign.ron` lists the level files in play order, and `Levels` is built from it at startup.
- [ ] Playing every level behaves exactly as before: blocks, triggers, portals, pickups, side walls, backgrounds and win criteria all unchanged.
- [ ] A level file round-trips: deserialize then serialize then deserialize is stable.
- [ ] `cargo build` is clean and `cargo test` passes.

## Notes

- `LEVEL0` is the strongest round-trip case — it is the only level exercising
  trigger groups and a portal (`ZIR1`, `ZAA1`).
- The commented-out `LevelObstacle::ForceField` level in `main.rs` should move
  across too, kept out of `campaign.ron`. Card `c0001` needs it to verify the
  force-field impact mapping.
- `ron` and `serde` are already in the lock file transitively but need adding as
  direct dependencies.

## Log

- 2026-08-25 created from the e01 epic breakdown
