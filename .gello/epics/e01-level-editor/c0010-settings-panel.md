---
id: c0010
title: Level settings panel
status: ready
epic: e01
depends: [c0005]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T23:00:00
order: 100
---

## What

Edit the parts of a level that are not the grid: everything in `LevelDefinition`
except `targets` and the out-of-scope `obstacles`.

## Acceptance criteria

- [ ] The panel edits background asset, background scroll velocity, simultaneous balls, win criteria, global pickups, `default_wall_l` and `default_wall_r`.
- [ ] Changes apply to the in-memory level immediately and are included when it is saved.
- [ ] Every edited value round-trips through RON unchanged.
- [ ] Invalid or out-of-range input is rejected without crashing.

## Notes

- `time_limit` is an `Option<Duration>` that nothing currently reads; include it
  only if it is cheap.
- `global_pickups` is a `Vec<PickupType>` where `PickupType::Grabber` is never
  constructed today — the panel is the natural place to make it reachable.
- Free-floating `LevelObstacle`s are explicitly out of scope for this epic.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
