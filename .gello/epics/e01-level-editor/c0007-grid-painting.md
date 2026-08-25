---
id: c0007
title: Grid painting
status: ready
epic: e01
depends: [c0006]
created: 2026-08-25
updated: 2026-08-25
status-changed: 2026-08-25T22:59:53
order: 70
---

## What

Actually edit the grid: a brush describing what to place, and click or drag to
place and erase it.

The brush covers the full token: block type, behaviour, trigger type and trigger
group, plus an erase mode. `Z` / `BlockType::Obstacle` is a normal brush — despite
the name it is an ordinary grid cell that happens to be unbreakable, unrelated to
`LevelObstacle`.

## Acceptance criteria

- [ ] A brush resource holds block type, behaviour, trigger type and trigger group, plus an erase mode.
- [ ] Clicking a cell writes the current brush into the in-memory level.
- [ ] The erase brush clears a cell back to empty (`..`).
- [ ] `Z` / `BlockType::Obstacle` can be painted.
- [ ] Painted cells appear immediately as blocks in the editor view.
- [ ] Dragging paints a run of cells without a click per cell.
- [ ] The edited layout serializes to a token grid that `interpret_grid` parses back to the same blocks.

## Notes

- Trigger group is `0..=9`, encoded as the 4th character and only valid when a
  trigger type is present. A brush with a trigger type but no group is invalid.
- Drag painting needs to be one undo step later (`c0011`), so record a drag as a
  single edit from the start rather than per-cell.

## Log

- 2026-08-25 created from the e01 epic breakdown
- 2026-08-25 status → ready (app)
